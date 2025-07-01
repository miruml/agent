// internal crates
use crate::deserialize_error;
use crate::logs::LogLevel;

// external crates
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Agent {
    pub device_id: String,
    pub activated: bool,
    pub backend_base_url: String,
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

impl<'de> Deserialize<'de> for Agent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct DeserializeAgent {
            device_id: String,
            activated: Option<bool>,
            backend_base_url: Option<String>,
            log_level: Option<LogLevel>,
            config_instance_deployment_base_path: Option<String>,
        }

        let result = match DeserializeAgent::deserialize(deserializer) {
            Ok(agent) => agent,
            Err(e) => {
                error!("Error deserializing agent: {}", e);
                return Err(e);
            }
        };

        let default = Agent::default();

        Ok(Agent {
            device_id: result.device_id,
            activated: result
                .activated
                .unwrap_or_else(|| deserialize_error!("agent", "activated", default.activated)),
            backend_base_url: result.backend_base_url.unwrap_or_else(|| {
                deserialize_error!("agent", "backend_base_url", default.backend_base_url)
            }),
            log_level: result
                .log_level
                .unwrap_or_else(|| deserialize_error!("agent", "log_level", default.log_level)),
            config_instance_deployment_base_path: result
                .config_instance_deployment_base_path
                .unwrap_or_else(|| {
                    deserialize_error!(
                        "agent",
                        "config_instance_deployment_base_path",
                        default.config_instance_deployment_base_path
                    )
                }),
        })
    }
}
