// external crates
use crate::filesys::{file::File, path::PathExt};
use crate::models::agent::Agent;
use crate::storage::errors::{AgentNotActivatedErr, StorageErr, StorageFileSysErr};
use crate::trace;

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
