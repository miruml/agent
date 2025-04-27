// external crates
use serde::{Deserialize, Serialize};
use crate::filesys::{file::File, path::PathExt};
use crate::storage::errors::{
    StorageErr,
    StorageFileSysErr,
    AgentNotActivatedErr,
};
use crate::trace;

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub client_id: String,
    pub activated: bool,
}

pub async fn assert_activated(agent_file: &File) -> Result<(), StorageErr> {
    // check the agent file exists
    agent_file.assert_exists().map_err(|e| StorageErr::FileSysErr(StorageFileSysErr { source: e, trace: trace!() }))?;

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