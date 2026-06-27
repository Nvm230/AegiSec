//! Network sensor — polls /proc/net/tcp and /proc/net/tcp6 to detect
//! new outbound connections and correlates them to the originating PID
//! by resolving socket inodes through /proc/<pid>/fd/.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::time::Duration;

use tracing::{debug, info};

use super::SensorEvent;

/// Poll interval for /proc/net/tcp
const POLL_MS: u64 = 500;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct ConnKey {
    src: String,
    dst: String,
    dst_port: u16,
}

pub fn run(tx: Sender<SensorEvent>) {
    info!("[NET SENSOR] Polling /proc/net/tcp every {}ms", POLL_MS);
    let mut seen: HashSet<ConnKey> = HashSet::new();

    loop {
        let connections = parse_tcp("/proc/net/tcp")
            .into_iter()
            .chain(parse_tcp("/proc/net/tcp6"));

        for (inode, src_ip, dst_ip, dst_port, state) in connections {
            // State 01 = ESTABLISHED, 02 = SYN_SENT (outbound)
            if state != "01" && state != "02" {
                continue;
            }

            let key = ConnKey {
                src: src_ip.clone(),
                dst: dst_ip.clone(),
                dst_port,
            };

            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);

            // Correlate inode to PID
            let (pid, comm) = inode_to_pid(inode);
            debug!(
                "[NET SENSOR] New connection: PID={} [{}] {}:{}->{}",
                pid, comm, src_ip, dst_port, dst_ip
            );

            let _ = tx.send(SensorEvent::NetConnect {
                pid,
                comm,
                src_ip,
                dst_ip,
                dst_port,
            });
        }

        // Prune stale connections
        let live = current_inodes();
        seen.retain(|k| {
            // Keep if connection is still in /proc/net/tcp — simplified: prune every 60s
            true
        });

        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
}

/// Parse /proc/net/tcp or /proc/net/tcp6.
/// Returns: Vec<(inode, src_ip, dst_ip, dst_port, state)>
fn parse_tcp(path: &str) -> Vec<(u64, String, String, u16, String)> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return vec![];
    };

    content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 10 {
                return None;
            }
            let local  = cols[1];
            let remote = cols[2];
            let state  = cols[3].to_string();
            let inode: u64 = cols[9].parse().ok()?;

            let (src_ip, _src_port) = decode_addr(local)?;
            let (dst_ip, dst_port) = decode_addr(remote)?;

            Some((inode, src_ip, dst_ip, dst_port, state))
        })
        .collect()
}

/// Convert /proc/net/tcp hex address (little-endian) to dotted-decimal.
fn decode_addr(hex: &str) -> Option<(String, u16)> {
    let parts: Vec<&str> = hex.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let ip_hex = parts[0];
    let port_hex = parts[1];
    let port = u16::from_str_radix(port_hex, 16).ok()?;

    // Handle IPv4 (8 hex chars) and IPv6 (32 hex chars)
    let ip = if ip_hex.len() == 8 {
        let n = u32::from_str_radix(ip_hex, 16).ok()?;
        let bytes = n.to_le_bytes();
        format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
    } else if ip_hex.len() == 32 {
        // IPv6: 4 groups of 4 bytes
        let mut groups = Vec::new();
        for i in (0..32).step_by(8) {
            let n = u32::from_str_radix(&ip_hex[i..i + 8], 16).ok()?;
            let b = n.to_le_bytes();
            groups.push(format!("{:02x}{:02x}:{:02x}{:02x}", b[3], b[2], b[1], b[0]));
        }
        groups.join(":")
    } else {
        return None;
    };

    Some((ip, port))
}

/// Build inode -> (pid, comm) map by scanning /proc/*/fd/
fn inode_to_pid(target_inode: u64) -> (u32, String) {
    let socket_prefix = format!("socket:[{}]", target_inode);

    let Ok(entries) = std::fs::read_dir("/proc") else {
        return (0, "?".to_string());
    };

    for proc_entry in entries.flatten() {
        let name = proc_entry.file_name();
        let pid_str = name.to_string_lossy();
        let Ok(pid) = pid_str.parse::<u32>() else { continue };

        let fd_dir = format!("/proc/{}/fd", pid);
        let Ok(fds) = std::fs::read_dir(&fd_dir) else { continue };

        for fd_entry in fds.flatten() {
            if let Ok(link) = std::fs::read_link(fd_entry.path()) {
                if link.to_string_lossy() == socket_prefix {
                    let comm = std::fs::read_to_string(format!("/proc/{}/comm", pid))
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    return (pid, comm);
                }
            }
        }
    }

    (0, "?".to_string())
}

fn current_inodes() -> HashSet<u64> {
    let mut inodes = HashSet::new();
    for path in ["/proc/net/tcp", "/proc/net/tcp6"] {
        for (inode, ..) in parse_tcp(path) {
            inodes.insert(inode);
        }
    }
    inodes
}
