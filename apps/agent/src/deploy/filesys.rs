// internal crates
use crate::crud::prelude::Read;
use crate::deploy::errors::{
    DeployErr,
    DeployFileSysErr,
    DeployCrudErr,
};
use crate::deploy::fsm;
use crate::deploy::observer::{Observer, on_update};
use crate::filesys::dir::Dir;
use crate::models::config_instance::{
    ConfigInstance,
    ConfigInstanceID,
};
use crate::trace;

// external crates
use tracing::{error, info};


pub async fn deploy_with_rollback<R>(
    to_remove: Vec<ConfigInstance>,
    to_deploy: Vec<ConfigInstance>,
    cfg_inst_data_reader: &R,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (
    Vec<ConfigInstance>,
    Vec<ConfigInstance>,
    Result<(), DeployErr>
)
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    // remove the previous instance. Don't worry whether it failed or not as we want
    // to attempt to deploy the next instance regardless
    let (to_remove, result) = remove_many(
        to_remove, deployment_dir, settings, observers
    ).await;
    if let Err(e) = result {
        error!("Error removing config instances: {:?}", e);
    }

    // deploy the new instance
    let (to_deploy, result) = deploy_many(
        to_deploy, cfg_inst_data_reader, deployment_dir, settings, observers
    ).await;
    if let Err(e) = result {
        error!("Error deploying config instances: {:?}", e);
    }

    // deployment FAILED -> rollback to the previous instance

    // remove the attempted deployment. Don't worry whether it failed or not as there
    // is nothing to do at this point. It will be attempted again with a retry.
    let (to_remove, result) = remove_many(
        to_remove, deployment_dir, settings, observers
    ).await;
    if let Err(e) = result {
        error!("Error stopping deployment: {:?}", e);
    }

    // deploy the previous instance. Don't worry whether it failed or not as there
    // is nothing to do at this point. It will be attempted again with a retry next
    // time.
    let (to_deploy, result) = deploy_many(
        to_deploy, cfg_inst_data_reader, deployment_dir, settings, observers
    ).await;
    if let Err(e) = result {
        error!("Error booting deployment: {:?}", e);
    }

    (to_remove, to_deploy, Ok(()))
}

// =================================== DEPLOY ====================================== //
async fn deploy_many<R>(
    instances: Vec<ConfigInstance>,
    data_fetcher: &R,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (Vec<ConfigInstance>, Result<(), DeployErr>)
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    let mut post_deploy_instances= Vec::new();
    for instance in instances {
        let (post_deploy_instance, result) = deploy(
            instance, data_fetcher, deployment_dir, settings, observers
        ).await;
        if let Err(e) = result {
            error!("Error deploying config instance '{}': {:?}", post_deploy_instance.id, e);
            return (post_deploy_instances, Err(e));
        }
        post_deploy_instances.push(post_deploy_instance);
    }
    (post_deploy_instances, Ok(()))
}

async fn deploy<R>(
    mut instance: ConfigInstance,
    data_fetcher: &R,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (ConfigInstance, Result<(), DeployErr>)
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    let result = write_instance_to_deployment_dir(
        &instance, data_fetcher, deployment_dir
    ).await;

    match result {
        Ok(_) => {
            info!("Deployed config instance '{}' to filesystem", instance.id);
            instance = fsm::deploy(instance);
            if let Err(e) = on_update(observers, &instance).await {
                return (instance, Err(e));
            }
            (instance, Ok(()))
        }
        Err(e) => {
            error!("Error deploying config instance '{}': {:?}", instance.id, e);
            instance = fsm::error(instance, settings, &e);
            if let Err(e) = on_update(observers, &instance).await {
                return (instance, Err(e));
            }
            (instance, Err(e))
        }
    }
}

async fn write_instance_to_deployment_dir<R>(
    instance: &ConfigInstance,
    data_fetcher: &R,
    deployment_dir: &Dir,
) -> Result<(), DeployErr>
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    // only write the config instance to the filesystem if it has a filepath
    let filepath = match &instance.filepath {
        Some(filepath) => filepath,
        None => return Ok(()),
    };

    let data = data_fetcher.read(instance.id.clone()).await.map_err(|e| {
        DeployErr::CrudErr(DeployCrudErr {
            source: e,
            trace: trace!(),
        })
    })?;

    let dest_file = deployment_dir.file(filepath);
    dest_file
        .write_json(&data, true, true)
        .await
        .map_err(|e| {
            DeployErr::FileSysErr(DeployFileSysErr {
                source: e,
                trace: trace!(),
            })
        })
}

// =================================== REMOVE ====================================== //
async fn remove_many(
    instances: Vec<ConfigInstance>,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (Vec<ConfigInstance>, Result<(), DeployErr>)
{
    let mut post_remove_instances = Vec::new();
    for instance in instances {
        let (post_remove_instance, result) = remove(instance, deployment_dir, settings, observers).await;
        if let Err(e) = result {
            error!("Error removing config instance '{}': {:?}", post_remove_instance.id, e);
            return (post_remove_instances, Err(e));
        }
        post_remove_instances.push(post_remove_instance);
    }
    (post_remove_instances, Ok(()))
}

async fn remove(
    mut instance: ConfigInstance,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (ConfigInstance, Result<(), DeployErr>)
{
    let result = delete_instance_from_deployment_dir(
        &instance, deployment_dir
    ).await;

    match result {
        Ok(_) => {
            info!(
                "Removed config instance '{}' from filesystem",
                instance.id
            );
            instance = fsm::remove(instance);
            if let Err(e) = on_update(observers, &instance).await {
                return (instance, Err(e));
            }
            (instance, Ok(()))
        }
        Err(e) => {
            error!(
                "Error removing config instance '{}': {:?}",
                instance.id, e
            );
            instance = fsm::error(instance, settings, &e);
            if let Err(e) = on_update(observers, &instance).await {
                return (instance, Err(e));
            }
            (instance, Err(e))
        }
    }
}

async fn delete_instance_from_deployment_dir(
    instance: &ConfigInstance,
    deployment_dir: &Dir,
) -> Result<(), DeployErr> {
    // only delete the config instance from the filesystem if it has a filepath
    let filepath = match &instance.filepath {
        Some(filepath) => filepath,
        None => return Ok(()),
    };

    let dest_file = deployment_dir.file(filepath);
    dest_file.delete().await.map_err(|e| {
        DeployErr::FileSysErr(DeployFileSysErr {
            source: e,
            trace: trace!(),
        })
    })
}
