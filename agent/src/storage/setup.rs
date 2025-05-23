// internal crates
use crate::crypt::rsa;
use crate::storage::{
    agent::Agent,
    errors::{StorageCryptErr, StorageErr, StorageFileSysErr},
    layout::StorageLayout,
};
use crate::trace;

pub async fn setup_storage(layout: &StorageLayout, agent: &Agent) -> Result<(), StorageErr> {
    // create the agent file
    let agent_file = layout.agent_file();
    agent_file
        .write_json(&agent, true, true)
        .await
        .map_err(|e| {
            StorageErr::FileSysErr(StorageFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;

    // create the auth directory
    let auth_dir = layout.auth_dir();
    auth_dir.root.create_if_absent().await.map_err(|e| {
        StorageErr::FileSysErr(StorageFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // generate the public and private keys
    let private_key_file = auth_dir.private_key_file();
    let public_key_file = auth_dir.public_key_file();
    rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
        .await
        .map_err(|e| {
            StorageErr::CryptErr(StorageCryptErr {
                source: e,
                trace: trace!(),
            })
        })?;

    Ok(())
}
