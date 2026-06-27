//! AegiSec Agent v2.0 — Main orchestrator
//! Wires all modules: sensors → DRS → isolator → telemetry

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod config;
mod drs;
mod isolator;
mod sensor;
mod telemetry;

use config::AgentConfig;
use drs::{ContainmentAction, DrsEngine};
use sensor::SensorEvent;
use telemetry::{build_http_client, build_payload, TelemetryClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("aegisec_agent=info,warn")),
        )
        .init();

    info!("╔══════════════════════════════════════╗");
    info!("║  🛡️  AegiSec Agent v2.0              ║");
    info!("║  Dynamic Risk Scoring Engine         ║");
    info!("╚══════════════════════════════════════╝");

    // Verify we're running as root (required for Netlink + iptables)
    if !nix::unistd::Uid::effective().is_root() {
        error!("AegiSec Agent must run as root (eBPF/Netlink/iptables require root).");
        std::process::exit(1);
    }

    // Load configuration
    let cfg = AgentConfig::load()?;
    info!("Agent ID  : {}", cfg.agent_id);
    info!("Hostname  : {}", cfg.hostname);
    info!("Backend   : {}", cfg.backend_url);
    info!("Cert Dir  : {}", cfg.cert_dir);

    // Build DRS engine with shared state
    let drs = Arc::new(DrsEngine::new(cfg.thresholds.clone()));

    // Spawn background decay loop
    DrsEngine::spawn_decay_loop(Arc::clone(&drs.table), cfg.thresholds.clone());

    // Build HTTP client
    let http_client = build_http_client(&cfg);

    // Build dual-channel telemetry client
    let (telem_client, priority_rx, normal_rx) = TelemetryClient::new();
    let telem_client = Arc::new(telem_client);

    // Spawn telemetry senders
    telemetry::spawn_priority_sender(priority_rx, http_client.clone(), cfg.backend_url.clone());
    telemetry::spawn_batch_sender(normal_rx, http_client.clone(), cfg.backend_url.clone(), cfg.batch_interval_secs);
    telemetry::spawn_heartbeat(http_client.clone(), cfg.backend_url.clone(), cfg.clone());

    // Unified sensor event channel (all sensors → DRS)
    let (sensor_tx, mut sensor_rx) = mpsc::channel::<SensorEvent>(8192);

    // ── Spawn Sensor Threads ──────────────────────────────────────────────────

    // 1. Process sensor (Netlink proc connector or procfs fallback)
    {
        let tx = sensor_tx.clone();
        let (std_tx, std_rx) = std::sync::mpsc::channel::<SensorEvent>();
        std::thread::spawn(move || sensor::process::run(std_tx));
        // Bridge std::sync::mpsc -> tokio mpsc
        let tok_tx = tx;
        tokio::task::spawn_blocking(move || {
            for ev in std_rx {
                if tok_tx.blocking_send(ev).is_err() {
                    break;
                }
            }
        });
    }

    // 2. File Integrity Monitor (inotify)
    {
        let tx = sensor_tx.clone();
        let (std_tx, std_rx) = std::sync::mpsc::channel::<SensorEvent>();
        std::thread::spawn(move || sensor::fim::run(std_tx));
        let tok_tx = tx;
        tokio::task::spawn_blocking(move || {
            for ev in std_rx {
                if tok_tx.blocking_send(ev).is_err() {
                    break;
                }
            }
        });
    }

    // 3. Network sensor (/proc/net/tcp polling)
    {
        let tx = sensor_tx.clone();
        let (std_tx, std_rx) = std::sync::mpsc::channel::<SensorEvent>();
        std::thread::spawn(move || sensor::network::run(std_tx));
        let tok_tx = tx;
        tokio::task::spawn_blocking(move || {
            for ev in std_rx {
                if tok_tx.blocking_send(ev).is_err() {
                    break;
                }
            }
        });
    }

    info!("✅ All sensors active. Monitoring...");
    info!("─────────────────────────────────────");

    // ── Main Event Processing Loop ────────────────────────────────────────────
    while let Some(event) = sensor_rx.recv().await {
        // Score the event in the DRS engine
        let result = match drs.score(&event).await {
            Some(r) => r,
            None => continue, // Event below noise floor
        };

        if result.delta == 0.0 {
            continue;
        }

        // Build telemetry payload
        let payload = build_payload(&cfg, &event, &result);

        // Execute containment if action was triggered
        let is_containment = result.action.is_some();
        if let Some(ref action) = result.action {
            match action {
                ContainmentAction::AlertOnly => {}
                _ => {
                    info!(
                        "[RESPONSE] Score={:.0} Action={} Entity={}",
                        result.score, payload.action_taken, result.entity
                    );
                    isolator::execute(action).await;
                }
            }
        }

        // Route to appropriate telemetry channel
        if is_containment {
            // High-priority: bypass buffer for containment events
            telem_client.send_priority(payload).await;
        } else {
            // Normal: batched
            telem_client.send_normal(payload).await;
        }
    }

    warn!("Sensor channel closed — agent shutting down.");
    Ok(())
}
