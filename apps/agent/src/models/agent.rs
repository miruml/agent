// internal crates
use crate::logs::LogLevel;

// external crates
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    pub device_id: String,
    pub activated: bool,
    pub backend_base_url: String,
    #[serde(default)]
    pub log_level: LogLevel,
    pub config_instance_deployment_base_path: String,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            device_id: "placeholder".to_string(),
            activated: false,
            backend_base_url: "https://configs.api.miruml.com/agent/v1".to_string(),
            log_level: LogLevel::Info,
            config_instance_deployment_base_path: "/srv/miru/configs/".to_string(),
        }
    }
}
