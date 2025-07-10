// internal crates
use crate::deserialize_error;
use crate::filesys::{file::File, path::PathExt};
use crate::storage::errors::{AgentNotActivatedErr, StorageErr, StorageFileSysErr};
use crate::trace;

// external crates
use serde::{Deserialize, Serialize};
use tracing::error;

pub async fn assert_activated(agent_file: &File) -> Result<(), StorageErr> {
    // check the agent file exists
    agent_file.assert_exists().map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // attempt to read it
    let agent = agent_file.read_json::<Agent>().await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // check the agent is activated
    if !agent.activated {
        return Err(StorageErr::AgentNotActivatedErr(Box::new(
            AgentNotActivatedErr {
                msg: "agent is not activated".to_string(),
                trace: trace!(),
            },
        )));
    }

    Ok(())
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Agent {
    pub device_id: String,
    pub activated: bool,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            device_id: "placeholder".to_string(),
            activated: false,
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
        })
    }
}
