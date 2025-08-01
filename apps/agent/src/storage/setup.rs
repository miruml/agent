// internal crates
use crate::auth::token::Token;
use crate::filesys::file::File;
use crate::storage::agent::Agent;
use crate::storage::settings::Settings;
use crate::storage::{
    errors::*,
    layout::StorageLayout,
};
use crate::trace;

pub async fn clean_storage_setup(
    layout: &StorageLayout,
    agent: &Agent,
    settings: &Settings,
    private_key_file: &File,
    public_key_file: &File,
) -> Result<(), StorageErr> {

    // overwrite the agent file
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

    // overwrite the settings file
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

    // overwrite the auth file
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

    // move the private and public keys to the auth directory
    private_key_file.move_to(&auth_dir.private_key_file(), true).await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;
    public_key_file.move_to(&auth_dir.public_key_file(), true).await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    // wipe the config instance deployment directory
    let config_instance_deployment_dir = layout.config_instance_deployment_dir();
    config_instance_deployment_dir.delete().await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;
    let config_instance_deployment_dir = layout.config_instance_deployment_dir();
    config_instance_deployment_dir
        .create_if_absent()
        .await
        .map_err(|e| {
            StorageErr::FileSysErr(Box::new(StorageFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    // wipe the cache
    let caches_dir = layout.caches_dir();
    caches_dir.delete().await.map_err(|e| {
        StorageErr::FileSysErr(Box::new(StorageFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    Ok(())
}
