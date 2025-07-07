// internal crates
use crate::auth::token::Token;
use crate::crypt::rsa;
use crate::storage::agent::Agent;
use crate::storage::settings::Settings;
use crate::storage::{
    errors::{StorageCryptErr, StorageErr, StorageFileSysErr},
    layout::StorageLayout,
};
use crate::trace;

pub async fn setup_storage(
    layout: &StorageLayout,
    agent: &Agent,
    settings: &Settings,
) -> Result<(), StorageErr> {
    // create the agent file
    let agent_file = layout.agent_file();
    agent_file
        .write_json(&agent, true, true)
        .await
        .map_err(|e| {
            StorageErr::FileSysErr(Box::new(StorageFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    // create the settings file
    let settings_file = layout.settings_file();
    settings_file
        .write_json(&settings, true, true)
        .await
        .map_err(|e| {
            StorageErr::FileSysErr(Box::new(StorageFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    // create the auth directory
    let auth_dir = layout.auth_dir();
    auth_dir.root.create_if_absent().await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // initialize the auth file with invalid authentication so that the agent doesn't
    // use old authentication by accident
    let token = Token::default();
    let auth_file = auth_dir.token_file();
    auth_file
        .write_json(&token, true, true)
        .await
        .map_err(|e| {
            StorageErr::FileSysErr(Box::new(StorageFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    // generate the public and private keys
    let private_key_file = auth_dir.private_key_file();
    let public_key_file = auth_dir.public_key_file();
    rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
        .await
        .map_err(|e| {
            StorageErr::CryptErr(Box::new(StorageCryptErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    // delete any lingering cache files so that the agent doesn't use old cache data
    let caches_dir = layout.caches_dir();
    caches_dir.delete().await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    Ok(())
}
