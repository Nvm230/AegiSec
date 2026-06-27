//! Agent configuration — loaded from YAML or environment variables.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique identifier for this agent instance.
    pub agent_id: String,
    pub hostname: String,
    pub os_info: String,

    /// Backend URL (e.g. https://192.168.1.100:8443)
    pub backend_url: String,

    /// Directory containing ca.crt, agent.crt, agent.key
    pub cert_dir: String,

    /// DRS threshold configuration
    pub thresholds: Thresholds,

    /// Telemetry batching interval in seconds
    pub batch_interval_secs: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thresholds {
    /// Score at which entity becomes SUSPICIOUS
    pub suspicious: f64,
    /// Score at which entity becomes BLOCKED (containment triggers)
    pub blocked: f64,
    /// Points subtracted per minute during decay
    pub decay_per_min: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            suspicious: 40.0,
            blocked: 80.0,
            decay_per_min: 5.0,
        }
    }
}

impl AgentConfig {
    pub fn load() -> Result<Self> {
        // Try to load from /etc/aegisec/config.yaml first
        if let Ok(content) = std::fs::read_to_string("/etc/aegisec/config.yaml") {
            if let Ok(cfg) = serde_yaml::from_str::<AgentConfig>(&content) {
                return Ok(cfg);
            }
        }

        // Fallback: build from environment / defaults
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(Self {
            agent_id: Uuid::new_v4().to_string(),
            hostname,
            os_info: std::env::consts::OS.to_string(),
            backend_url: std::env::var("AEGISEC_BACKEND")
                .unwrap_or_else(|_| "https://localhost:8443".to_string()),
            cert_dir: std::env::var("AEGISEC_CERT_DIR")
                .unwrap_or_else(|_| "/etc/aegisec/certs".to_string()),
            thresholds: Thresholds::default(),
            batch_interval_secs: 10,
        })
    }
}
