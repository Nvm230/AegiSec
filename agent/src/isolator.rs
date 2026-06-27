//! Containment module — kills process trees and applies firewall rules.

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use tracing::{error, info, warn};

use crate::drs::ContainmentAction;

/// Execute a containment action.
pub async fn execute(action: &ContainmentAction) {
    match action {
        ContainmentAction::KillProcess { pid, uid } => {
            kill_tree(*pid);
            block_by_uid(*uid).await;
        }
        ContainmentAction::BlockIp { ip, uid } => {
            block_ip_outbound(ip).await;
            if *uid > 0 {
                block_by_uid(*uid).await;
            }
        }
        ContainmentAction::AlertOnly => {
            // Telemetry already queued — no active response
        }
    }
}

/// Recursively find all children of `pid` and kill bottom-up.
/// This prevents zombie processes and orphan attack tools.
fn kill_tree(pid: u32) {
    info!("[ISOLATOR] 🔪 Killing process tree rooted at PID={}", pid);

    // First collect all descendants
    let children = find_all_children(pid);

    // Kill children first (leaves), then root
    for child in children.iter().rev() {
        send_signal(*child, Signal::SIGSTOP); // Freeze first
        send_signal(*child, Signal::SIGKILL); // Then destroy
    }
    // Kill root
    send_signal(pid, Signal::SIGSTOP);
    send_signal(pid, Signal::SIGKILL);
}

/// Breadth-first traversal of the process tree.
fn find_all_children(root: u32) -> Vec<u32> {
    let mut result = vec![];
    let mut queue = vec![root];

    while let Some(parent) = queue.pop() {
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let pid_str = name.to_string_lossy();
                let Ok(pid) = pid_str.parse::<u32>() else { continue };

                // Read ppid from /proc/<pid>/stat
                let ppid = read_ppid(pid);
                if ppid == parent && pid != root {
                    result.push(pid);
                    queue.push(pid);
                }
            }
        }
    }

    result
}

fn read_ppid(pid: u32) -> u32 {
    std::fs::read_to_string(format!("/proc/{}/status", pid))
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("PPid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|p| p.parse::<u32>().ok())
        })
        .unwrap_or(0)
}

fn send_signal(pid: u32, sig: Signal) {
    match kill(Pid::from_raw(pid as i32), sig) {
        Ok(_) => info!("[ISOLATOR] {:?} sent to PID={}", sig, pid),
        Err(e) => warn!("[ISOLATOR] Cannot {:?} PID={}: {}", sig, pid, e),
    }
}

/// Block all OUTPUT traffic from a UID using iptables/nftables.
/// This cuts the attacker's C2 connection even if they escalated.
async fn block_by_uid(uid: u32) {
    if uid == 0 || uid > 65000 {
        return; // Don't block root or invalid UIDs
    }

    info!("[ISOLATOR] 🚫 Blocking UID={} outbound traffic", uid);

    // Try nftables first
    let nft_result = tokio::process::Command::new("nft")
        .args([
            "add", "rule", "inet", "filter", "output",
            "meta", "skuid", &uid.to_string(), "drop",
        ])
        .output()
        .await;

    if nft_result.map(|o| o.status.success()).unwrap_or(false) {
        info!("[ISOLATOR] ✅ UID={} blocked via nftables", uid);
        return;
    }

    // Fallback: iptables owner match
    let ipt_result = tokio::process::Command::new("iptables")
        .args(["-A", "OUTPUT", "-m", "owner", "--uid-owner", &uid.to_string(), "-j", "DROP"])
        .output()
        .await;

    match ipt_result {
        Ok(o) if o.status.success() => {
            info!("[ISOLATOR] ✅ UID={} blocked via iptables", uid);
        }
        Ok(o) => {
            error!(
                "[ISOLATOR] iptables failed for UID={}: {}",
                uid, String::from_utf8_lossy(&o.stderr)
            );
        }
        Err(e) => error!("[ISOLATOR] Cannot run iptables: {}", e),
    }
}

/// Block a specific destination IP on OUTPUT.
async fn block_ip_outbound(ip: &str) {
    info!("[ISOLATOR] 🚫 Blocking outbound to IP={}", ip);

    // nftables
    let nft = tokio::process::Command::new("nft")
        .args([
            "add", "rule", "inet", "filter", "output",
            "ip", "daddr", ip, "drop",
        ])
        .output()
        .await;

    if nft.map(|o| o.status.success()).unwrap_or(false) {
        info!("[ISOLATOR] ✅ IP={} blocked via nftables", ip);
        return;
    }

    // iptables fallback
    let _ = tokio::process::Command::new("iptables")
        .args(["-A", "OUTPUT", "-d", ip, "-j", "DROP"])
        .output()
        .await;
}
