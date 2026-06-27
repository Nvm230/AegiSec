//! Dynamic Risk Scoring engine.
//! Maintains per-entity scores in a shared HashMap.
//! Decay loop runs every 60s in a background Tokio task.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

use crate::config::Thresholds;
use crate::sensor::SensorEvent;

/// Lifecycle state of a monitored entity.
#[derive(Debug, Clone, PartialEq)]
pub enum EntityState {
    Clean,
    Suspicious,
    Blocked,
}

/// Per-entity risk profile stored in the DRS table.
#[derive(Debug, Clone)]
pub struct RiskProfile {
    pub score: f64,
    pub state: EntityState,
    pub last_event: Instant,
    /// Recent process names seen (for H-02 velocity detection)
    pub recent_comms: Vec<(Instant, String)>,
    /// PID associated with this entity (for containment)
    pub last_pid: u32,
    /// UID of this entity
    pub uid: u32,
}

impl RiskProfile {
    fn new(uid: u32, pid: u32) -> Self {
        Self {
            score: 0.0,
            state: EntityState::Clean,
            last_event: Instant::now(),
            recent_comms: Vec::new(),
            last_pid: pid,
            uid,
        }
    }
}

/// Shared DRS state accessible by multiple async tasks.
pub type DrsTable = Arc<Mutex<HashMap<String, RiskProfile>>>;

/// Result of scoring a single event.
pub struct ScoreResult {
    pub entity: String,
    pub score: f64,
    pub delta: f64,
    pub action: Option<ContainmentAction>,
    pub state: EntityState,
}

#[derive(Debug, Clone)]
pub enum ContainmentAction {
    KillProcess { pid: u32, uid: u32 },
    BlockIp { ip: String, uid: u32 },
    AlertOnly,
}

/// Blacklisted commands that score on any occurrence.
const BLACKLIST_DIRECT: &[&str] = &[
    "nc", "netcat", "ncat", "socat",   // Netcat variants
    "msfconsole", "msfvenom",           // Metasploit
    "base64",                           // Payload encoding
    "curl", "wget",                     // Download
];

/// Enumeration commands (score via velocity).
const ENUM_CMDS: &[&str] = &[
    "whoami", "id", "uname", "getcap", "getfacl",
    "netstat", "ss", "lsof", "ps", "hostname",
    "ip", "ifconfig", "find", "env", "printenv",
    "cat", "awk", "grep",
];

/// High-value single commands.
const HIGH_VALUE: &[&str] = &["getcap", "sudo", "su", "passwd", "chpasswd", "visudo"];

/// Web server process names — if these spawn shells it's a web shell.
const WEB_PARENTS: &[&str] = &["apache", "httpd", "nginx", "tomcat", "php", "php-fpm", "gunicorn", "lighttpd"];
const SHELL_NAMES: &[&str] = &["sh", "bash", "dash", "zsh", "csh", "tcsh"];

/// Scripting parents that may spawn reverse shells.
const SCRIPT_PARENTS: &[&str] = &["python", "python3", "python2", "perl", "ruby", "php"];
const REVERSE_KW: &[&str] = &["pty.spawn", "socket", "dup2", "/dev/tcp", "/dev/udp", "os.system", "-e /bin", "-c bash", "bash -i"];

/// Suspicious outbound ports.
const SUSP_PORTS: &[u16] = &[4444, 1337, 9001, 31337, 4445, 12345, 6666, 7777, 8888];

pub struct DrsEngine {
    pub table: DrsTable,
    thresholds: Thresholds,
}

impl DrsEngine {
    pub fn new(thresholds: Thresholds) -> Self {
        Self {
            table: Arc::new(Mutex::new(HashMap::new())),
            thresholds,
        }
    }

    /// Score an event and return the result + optional containment action.
    pub async fn score(&self, event: &SensorEvent) -> Option<ScoreResult> {
        let mut table = self.table.lock().await;

        match event {
            SensorEvent::ProcessSpawn { pid, ppid, uid, comm, cmdline } => {
                let entity = format!("uid_{}", uid);
                let profile = table.entry(entity.clone()).or_insert_with(|| RiskProfile::new(*uid, *pid));
                profile.last_pid = *pid;
                profile.last_event = Instant::now();

                let comm_lc = comm.to_lowercase();
                let cmd_lc = cmdline.to_lowercase();

                let mut delta = 0.0_f64;
                let mut action = None;

                // H-01: Web Shell — web parent spawning a shell
                let is_web_parent = {
                    let ppid_comm = std::fs::read_to_string(format!("/proc/{}/comm", ppid))
                        .unwrap_or_default().to_lowercase();
                    WEB_PARENTS.iter().any(|p| ppid_comm.contains(p))
                };
                if is_web_parent && SHELL_NAMES.contains(&comm_lc.as_str()) {
                    delta += 100.0;
                    action = Some(ContainmentAction::KillProcess { pid: *pid, uid: *uid });
                    warn!("[H-01] 🚨 Web Shell detected: PID={} comm={} parent={}", pid, comm, ppid);
                }

                // H-05: Reverse shell — scripting parent + shell + suspicious keywords
                let is_script_parent = {
                    let ppid_comm = std::fs::read_to_string(format!("/proc/{}/comm", ppid))
                        .unwrap_or_default().to_lowercase();
                    SCRIPT_PARENTS.iter().any(|p| ppid_comm.contains(p))
                };
                if is_script_parent && SHELL_NAMES.contains(&comm_lc.as_str())
                    && REVERSE_KW.iter().any(|k| cmd_lc.contains(k))
                {
                    delta += 100.0;
                    action = Some(ContainmentAction::KillProcess { pid: *pid, uid: *uid });
                    warn!("[H-05] 🚨 Reverse Shell detected: PID={} cmd={}", pid, cmdline);
                }

                // H-00: Direct blacklist match
                if BLACKLIST_DIRECT.contains(&comm_lc.as_str()) {
                    delta += 40.0;
                    warn!("[H-00] ⚠️  Blacklisted binary: {} PID={}", comm, pid);
                }

                // H-02: Enumeration velocity
                if ENUM_CMDS.contains(&comm_lc.as_str()) {
                    if HIGH_VALUE.contains(&comm_lc.as_str()) {
                        delta += 20.0;
                        info!("[H-02] High-value tool: {}", comm);
                    }

                    let now = Instant::now();
                    profile.recent_comms.retain(|(t, _)| now.duration_since(*t) < Duration::from_secs(60));
                    profile.recent_comms.push((now, comm.clone()));
                    let distinct_count = {
                        let s: std::collections::HashSet<&str> =
                            profile.recent_comms.iter().map(|(_, c)| c.as_str()).collect();
                        s.len()
                    };
                    if distinct_count >= 3 {
                        delta += 50.0;
                        profile.recent_comms.clear();
                        warn!("[H-02] Enumeration pattern: {} distinct tools in 60s", distinct_count);
                    }
                }

                if delta > 0.0 {
                    self.apply_score(profile, delta, &mut action);
                    let result = ScoreResult {
                        entity, score: profile.score, delta,
                        action, state: profile.state.clone(),
                    };
                    return Some(result);
                }
            }

            SensorEvent::NetConnect { pid, comm, dst_ip, dst_port, src_ip: _ } => {
                let entity = format!("uid_{}", self.pid_to_uid(*pid).await);
                let uid = self.pid_to_uid(*pid).await;
                let profile = table.entry(entity.clone()).or_insert_with(|| RiskProfile::new(uid, *pid));
                profile.last_pid = *pid;
                profile.last_event = Instant::now();

                let mut delta = 0.0_f64;
                let mut action = None;

                // H-04: Suspicious outbound port
                if SUSP_PORTS.contains(dst_port) {
                    delta = 85.0;
                    action = Some(ContainmentAction::BlockIp { ip: dst_ip.clone(), uid });
                    warn!("[H-04] 🚨 Suspicious port {}: PID={} [{}] -> {}:{}", dst_port, pid, comm, dst_ip, dst_port);
                }

                if delta > 0.0 {
                    self.apply_score(profile, delta, &mut action);
                    return Some(ScoreResult {
                        entity, score: profile.score, delta, action, state: profile.state.clone(),
                    });
                }
            }

            SensorEvent::FileModified { path } => {
                // FIM events always alert regardless of entity score
                warn!("[FIM] Critical file modified: {}", path);
                return Some(ScoreResult {
                    entity: format!("fim:{}", path),
                    score: 90.0,
                    delta: 90.0,
                    action: Some(ContainmentAction::AlertOnly),
                    state: EntityState::Blocked,
                });
            }

            SensorEvent::AuthFailure { user, src_ip } => {
                let entity = src_ip.as_deref()
                    .map(|ip| format!("ip_{}", ip))
                    .unwrap_or_else(|| format!("user_{}", user));
                let profile = table.entry(entity.clone()).or_insert_with(|| RiskProfile::new(0, 0));
                profile.last_event = Instant::now();
                let mut delta = 15.0_f64;
                let mut action = None;
                self.apply_score(profile, delta, &mut action);
                if delta > 0.0 {
                    return Some(ScoreResult {
                        entity, score: profile.score, delta, action, state: profile.state.clone(),
                    });
                }
            }

            SensorEvent::ProcessExit { pid } => {
                // Clean up recent_comms for this pid (best effort).
                debug!("[DRS] PID {} exited", pid);
            }
        }

        None
    }

    fn apply_score(&self, profile: &mut RiskProfile, delta: f64, action: &mut Option<ContainmentAction>) {
        profile.score = (profile.score + delta).min(200.0);

        if profile.score >= self.thresholds.blocked {
            profile.state = EntityState::Blocked;
            if action.is_none() {
                *action = Some(ContainmentAction::KillProcess { pid: profile.last_pid, uid: profile.uid });
            }
        } else if profile.score >= self.thresholds.suspicious {
            profile.state = EntityState::Suspicious;
        }
    }

    async fn pid_to_uid(&self, pid: u32) -> u32 {
        std::fs::read_to_string(format!("/proc/{}/status", pid))
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("Uid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|u| u.parse::<u32>().ok())
            })
            .unwrap_or(0)
    }

    /// Spawn the background decay loop as a Tokio task.
    pub fn spawn_decay_loop(table: DrsTable, thresholds: Thresholds) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;
                let mut table = table.lock().await;
                let mut decayed = 0u32;
                for (entity, profile) in table.iter_mut() {
                    if profile.state == EntityState::Blocked {
                        continue; // Don't decay blocked entities automatically
                    }
                    let mins = profile.last_event.elapsed().as_secs_f64() / 60.0;
                    let decay = mins * thresholds.decay_per_min;
                    if decay > 0.0 {
                        let old = profile.score;
                        profile.score = (profile.score - decay).max(0.0);
                        if profile.score < thresholds.suspicious && profile.state == EntityState::Suspicious {
                            profile.state = EntityState::Clean;
                        }
                        if old > 0.0 {
                            decayed += 1;
                        }
                    }
                }
                // Remove entities with score=0 and no recent activity (> 10 min)
                table.retain(|_, p| {
                    p.score > 0.0 || p.last_event.elapsed() < Duration::from_secs(600)
                });
                info!("[DECAY] Applied decay to {} entities", decayed);
            }
        });
    }
}
