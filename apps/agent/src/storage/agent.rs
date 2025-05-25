// internal crates
use crate::logs::LogLevel;

// external crates
use crate::filesys::{file::File, path::PathExt};
use crate::storage::errors::{AgentNotActivatedErr, StorageErr, StorageFileSysErr};
use crate::trace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    pub device_id: String,
    pub activated: bool,
    pub backend_base_url: String,
    #[serde(default)]
    pub log_level: LogLevel,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            device_id: "placeholder".to_string(),
            activated: false,
            backend_base_url: "https://configs.api.miruml.com/agent/v1".to_string(),
            log_level: LogLevel::Info,
        }
    }
}

pub async fn assert_activated(agent_file: &File) -> Result<(), StorageErr> {
    // check the agent file exists
    agent_file.assert_exists().map_err(|e| {
        StorageErr::FileSysErr(StorageFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // attempt to read it
    let agent = agent_file.read_json::<Agent>().await.map_err(|e| {
        StorageErr::FileSysErr(StorageFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // check the agent is activated
    if !agent.activated {
        return Err(StorageErr::AgentNotActivatedErr(AgentNotActivatedErr {
            msg: "agent is not activated".to_string(),
            trace: trace!(),
        }));
    }

    Ok(())
}
