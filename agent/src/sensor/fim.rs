//! File Integrity Monitor — uses Linux inotify to watch critical paths.

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::Sender;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use tracing::{error, info, warn};

use super::SensorEvent;

/// Critical paths to monitor.
const WATCH_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "/etc/crontab",
    "/etc/cron.d",
    "/var/spool/cron",
    "/root/.ssh/authorized_keys",
];

/// User home pattern — we also watch all authorized_keys we can find.
const SSH_DIR_PATTERN: &str = "/home";

pub fn run(tx: Sender<SensorEvent>) {
    let inotify: Inotify = match Inotify::init(InitFlags::IN_CLOEXEC) {
        Ok(i) => i,
        Err(e) => {
            error!("[FIM] Failed to init inotify: {} — FIM disabled", e);
            return;
        }
    };

    let flags = AddWatchFlags::IN_MODIFY
        | AddWatchFlags::IN_ATTRIB
        | AddWatchFlags::IN_CREATE
        | AddWatchFlags::IN_DELETE
        | AddWatchFlags::IN_MOVED_TO
        | AddWatchFlags::IN_MOVED_FROM;

    // Build WatchDescriptor -> path map
    let mut wd_map: HashMap<_, String> = HashMap::new();

    for path in WATCH_PATHS {
        if Path::new(path).exists() {
            match inotify.add_watch(Path::new(path), flags) {
                Ok(wd) => {
                    info!("[FIM] Watching: {}", path);
                    wd_map.insert(wd, path.to_string());
                }
                Err(e) => warn!("[FIM] Cannot watch {}: {}", path, e),
            }
        }
    }

    // Also watch all /home/<user>/.ssh/authorized_keys
    if let Ok(home_entries) = std::fs::read_dir(SSH_DIR_PATTERN) {
        for entry in home_entries.flatten() {
            let auth_keys = entry.path().join(".ssh/authorized_keys");
            if auth_keys.exists() {
                if let Ok(wd) = inotify.add_watch(&auth_keys, flags) {
                    let p = auth_keys.to_string_lossy().to_string();
                    info!("[FIM] Watching: {}", p);
                    wd_map.insert(wd, p);
                }
            }
        }
    }

    info!("[FIM] ✅ Watching {} paths via inotify", wd_map.len());

    loop {
        match inotify.read_events() {
            Ok(events) => {
                for ev in events {
                    let path = wd_map
                        .get(&ev.wd)
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());

                    let name = ev
                        .name
                        .map(|n: std::ffi::OsString| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let full_path = if name.is_empty() {
                        path
                    } else {
                        format!("{}/{}", path, name)
                    };

                    warn!("[FIM] 🔔 Change detected: {}", full_path);
                    let _ = tx.send(SensorEvent::FileModified { path: full_path });
                }
            }
            Err(e) => {
                error!("[FIM] inotify read error: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}
