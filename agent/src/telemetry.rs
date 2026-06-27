//! Telemetry client with dual-channel queue:
//!  - PRIORITY: immediate POST for containment events (bypasses buffer)
//!  - NORMAL: buffered batch sent every N seconds
//!  - HEARTBEAT: ping every 30 seconds

use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::config::AgentConfig;
use crate::drs::{EntityState, ScoreResult};
use crate::sensor::SensorEvent;

/// JSON payload sent to the backend.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryPayload {
    pub agent_id: String,
    pub hostname: String,
    pub os_info: String,
    pub timestamp: String,
    pub event_type: String,
    pub process_name: Option<String>,
    pub cmdline: Option<String>,
    pub risk_score: f64,
    pub severity: String,
    pub action_taken: String,
}

/// Heartbeat payload
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HeartbeatPayload {
    agent_id: String,
    hostname: String,
    timestamp: String,
    status: String,
}

pub enum TelemetryMessage {
    Priority(TelemetryPayload),
    Normal(TelemetryPayload),
}

/// Build the telemetry client and return senders for both channels.
pub struct TelemetryClient {
    priority_tx: mpsc::Sender<TelemetryPayload>,
    normal_tx: mpsc::Sender<TelemetryPayload>,
}

impl TelemetryClient {
    pub fn new() -> (Self, mpsc::Receiver<TelemetryPayload>, mpsc::Receiver<TelemetryPayload>) {
        let (priority_tx, priority_rx) = mpsc::channel(256);
        let (normal_tx, normal_rx) = mpsc::channel(4096);
        (Self { priority_tx, normal_tx }, priority_rx, normal_rx)
    }

    pub async fn send_priority(&self, payload: TelemetryPayload) {
        if self.priority_tx.send(payload).await.is_err() {
            warn!("[TELEMETRY] Priority channel full");
        }
    }

    pub async fn send_normal(&self, payload: TelemetryPayload) {
        if self.normal_tx.send(payload).await.is_err() {
            warn!("[TELEMETRY] Normal channel full — dropping event");
        }
    }
}

/// Build an HTTP client with mTLS if certs are present.
pub fn build_http_client(cfg: &AgentConfig) -> reqwest::Client {
    let cert_dir = &cfg.cert_dir;
    let agent_cert_path = format!("{}/agent.crt", cert_dir);
    let agent_key_path  = format!("{}/agent.key", cert_dir);
    let ca_cert_path    = format!("{}/ca.crt", cert_dir);

    let builder = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .connection_verbose(false);

    if let (Ok(cert), Ok(key), Ok(ca)) = (
        std::fs::read(&agent_cert_path),
        std::fs::read(&agent_key_path),
        std::fs::read(&ca_cert_path),
    ) {
        let mut pem = cert;
        pem.push(b'\n');
        pem.extend_from_slice(&key);

        if let (Ok(identity), Ok(ca_cert)) = (
            reqwest::Identity::from_pem(&pem),
            reqwest::Certificate::from_pem(&ca),
        ) {
            info!("[TLS] mTLS enabled with cert from {}", cert_dir);
            return builder
                .identity(identity)
                .add_root_certificate(ca_cert)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
        }
    }

    warn!("[TLS] Running without mTLS (dev mode)");
    builder
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Spawn the priority sender task — sends immediately on receipt.
pub fn spawn_priority_sender(
    mut rx: mpsc::Receiver<TelemetryPayload>,
    client: reqwest::Client,
    endpoint: String,
) {
    tokio::spawn(async move {
        while let Some(payload) = rx.recv().await {
            let url = format!("{}/api/v1/events", endpoint);
            match client.post(&url).json(&payload).send().await {
                Ok(r) => {
                    if !r.status().is_success() {
                        warn!("[TELEMETRY PRIORITY] Backend returned {}", r.status());
                    } else {
                        info!("[TELEMETRY PRIORITY] ✅ Sent containment event: {}", payload.action_taken);
                    }
                }
                Err(e) => error!("[TELEMETRY PRIORITY] Failed to send: {}", e),
            }
        }
    });
}

/// Spawn the batched sender task — collects events and sends every `batch_secs`.
pub fn spawn_batch_sender(
    mut rx: mpsc::Receiver<TelemetryPayload>,
    client: reqwest::Client,
    endpoint: String,
    batch_secs: u64,
) {
    tokio::spawn(async move {
        let mut batch: Vec<TelemetryPayload> = Vec::new();
        let mut ticker = interval(Duration::from_secs(batch_secs));

        loop {
            tokio::select! {
                Some(payload) = rx.recv() => {
                    batch.push(payload);
                    // Flush early if batch is large
                    if batch.len() >= 50 {
                        flush_batch(&client, &endpoint, &mut batch).await;
                    }
                }
                _ = ticker.tick() => {
                    if !batch.is_empty() {
                        flush_batch(&client, &endpoint, &mut batch).await;
                    }
                }
            }
        }
    });
}

async fn flush_batch(client: &reqwest::Client, endpoint: &str, batch: &mut Vec<TelemetryPayload>) {
    let url = format!("{}/api/v1/events/batch", endpoint);
    match client.post(&url).json(&batch).send().await {
        Ok(r) if r.status().is_success() => {
            debug!("[TELEMETRY BATCH] Flushed {} events", batch.len());
        }
        Ok(r) => {
            warn!("[TELEMETRY BATCH] Backend returned {} — falling back to individual sends", r.status());
            send_individually(client, endpoint, batch).await;
        }
        Err(_) => {
            send_individually(client, endpoint, batch).await;
        }
    }
    batch.clear();
}

async fn send_individually(client: &reqwest::Client, endpoint: &str, batch: &[TelemetryPayload]) {
    let url = format!("{}/api/v1/events", endpoint);
    for payload in batch {
        let _ = client.post(&url).json(payload).send().await;
    }
}

/// Spawn heartbeat task — POSTs a ping every 30 seconds.
pub fn spawn_heartbeat(client: reqwest::Client, endpoint: String, cfg: AgentConfig) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        let url = format!("{}/api/v1/heartbeat", endpoint);
        loop {
            ticker.tick().await;
            let ping = HeartbeatPayload {
                agent_id: cfg.agent_id.clone(),
                hostname: cfg.hostname.clone(),
                timestamp: Utc::now().to_rfc3339(),
                status: "ACTIVE".to_string(),
            };
            match client.post(&url).json(&ping).send().await {
                Ok(_) => debug!("[HEARTBEAT] ✅ sent"),
                Err(e) => warn!("[HEARTBEAT] Failed: {}", e),
            }
        }
    });
}

/// Helper: build payload from sensor event + DRS result.
pub fn build_payload(
    cfg: &AgentConfig,
    event: &SensorEvent,
    result: &ScoreResult,
) -> TelemetryPayload {
    let (event_type, process_name, cmdline) = match event {
        SensorEvent::ProcessSpawn { comm, cmdline, .. } => {
            ("PROCESS_SPAWN".to_string(), Some(comm.clone()), Some(cmdline.clone()))
        }
        SensorEvent::NetConnect { comm, dst_ip, dst_port, .. } => {
            ("NET_CONNECT".to_string(), Some(comm.clone()), Some(format!("{}:{}", dst_ip, dst_port)))
        }
        SensorEvent::FileModified { path } => {
            ("FILE_MODIFIED".to_string(), None, Some(path.clone()))
        }
        SensorEvent::AuthFailure { user, src_ip } => {
            ("AUTH_FAILURE".to_string(), Some(user.clone()), src_ip.clone())
        }
        SensorEvent::ProcessExit { pid } => {
            ("PROCESS_EXIT".to_string(), None, Some(format!("pid={}", pid)))
        }
    };

    let severity = match result.state {
        EntityState::Blocked => "BLOCKED",
        EntityState::Suspicious => "SUSPICIOUS",
        EntityState::Clean => "INFO",
    };

    let action_taken = match &result.action {
        Some(crate::drs::ContainmentAction::KillProcess { pid, .. }) => format!("KILL_PROCESS:{}", pid),
        Some(crate::drs::ContainmentAction::BlockIp { ip, .. }) => format!("BLOCK_IP:{}", ip),
        Some(crate::drs::ContainmentAction::AlertOnly) => "ALERT_ONLY".to_string(),
        None => "NONE".to_string(),
    };

    TelemetryPayload {
        agent_id: cfg.agent_id.clone(),
        hostname: cfg.hostname.clone(),
        os_info: cfg.os_info.clone(),
        timestamp: Utc::now().to_rfc3339(),
        event_type,
        process_name,
        cmdline,
        risk_score: result.score,
        severity: severity.to_string(),
        action_taken,
    }
}
