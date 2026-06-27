//! Sensor module — unified event type and sensor spawning.
pub mod process;
pub mod network;
pub mod fim;

use serde::{Deserialize, Serialize};

/// Unified event type emitted by all sensors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorEvent {
    /// A new process was spawned (execve/fork intercepted via Netlink).
    ProcessSpawn {
        pid: u32,
        ppid: u32,
        uid: u32,
        comm: String,   // short comm (≤15 chars)
        cmdline: String,
    },
    /// A process exited (used to clean DRS state).
    ProcessExit {
        pid: u32,
    },
    /// Outbound TCP connection, correlated to a PID.
    NetConnect {
        pid: u32,
        comm: String,
        src_ip: String,
        dst_ip: String,
        dst_port: u16,
    },
    /// A critical file was modified (inotify).
    FileModified {
        path: String,
    },
    /// SSH/PAM auth failure parsed from /var/log/auth.log
    AuthFailure {
        user: String,
        src_ip: Option<String>,
    },
}
