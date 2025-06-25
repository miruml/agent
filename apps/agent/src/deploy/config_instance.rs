// internal crates
use crate::deploy::errors::{
    ConfigInstanceWithMismatchingFilepath,
    DeployErr,
    DeployFileSysErr,
    MismatchingFilepathErr,
};
use crate::filesys::dir::Dir;
use crate::fsm::config_instance as fsm;
use crate::models::config_instance::ConfigInstance;
use crate::trace;

// external crates
use tracing::{error, info};

pub trait Observer {
    fn on_update(&self, config_instance: &ConfigInstance) -> Result<(), DeployErr>;
}

pub async fn replace<O: Observer>(
    old: ConfigInstance,
    new: ConfigInstance,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observer: &O,
) -> Result<(ConfigInstance, ConfigInstance), DeployErr> {

    // double check that the filepath's are identical
    if old.filepath != new.filepath {
        return Err(DeployErr::MismatchingFilepathErr(MismatchingFilepathErr {
            old: ConfigInstanceWithMismatchingFilepath::from_instance(old),
            new: ConfigInstanceWithMismatchingFilepath::from_instance(new),
            trace: trace!(),
        }));
    }

    // remove the previous instance. Don't worry whether it failed or not as we want
    // to attempt to deploy the next instance regardless
    let (old, result) = remove(
        old, deployment_dir, settings, observer,
    ).await;
    if let Err(e) = result {
        error!("Error stopping deployment '{}': {:?}", old.id, e);
    }

    // deploy the new instance
    let (new, result) = deploy(
        new, deployment_dir, settings, observer,
    ).await;
    if let Err(e) = result {
        error!("Error booting deployment '{}': {:?}", new.id, e);
    } else {
        return Ok((new, old));
    }

    // deployment FAILED -> rollback to the previous instance 

    // remove the attempted deployment. Don't worry whether it failed or not as there
    // is nothing to do at this point. It will be attempted again with a retry.
    let (new, result) = remove(
        new, deployment_dir, settings, observer,
    ).await;
    if let Err(e) = result {
        error!("Error stopping deployment '{}': {:?}", new.id, e);
    }

    // deploy the previous instance. Don't worry whether it failed or not as there
    // is nothing to do at this point. It will be attempted again with a retry next
    // time.
    let (old, result) = deploy(
        old, deployment_dir, settings, observer,
    ).await;
    if let Err(e) = result {
        error!("Error booting deployment '{}': {:?}", old.id, e);
    }

    Ok((old, new))
}

// =================================== DEPLOY ====================================== //
pub async fn deploy<O: Observer>(
    mut instance: ConfigInstance,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observer: &O,
) -> (ConfigInstance, Result<(), DeployErr>) {

    let result = write_instance_to_deployment_dir(
        &instance, deployment_dir,
    ).await;

    match result {
        Ok(_) => {
            info!("Deployed config instance '{}' to filesystem", instance.id);
            fsm::deploy(&mut instance);
            if let Err(e) = observer.on_update(&instance) {
                return (instance, Err(e));
            }
            (instance, Ok(()))
        }
        Err(e) => {
            error!("Error deploying config instance '{}': {:?}", instance.id, e);
            fsm::error(&mut instance, settings, &e);
            if let Err(e) = observer.on_update(&instance) {
                return (instance, Err(e));
            }
            (instance, Err(e))
        }
    }

}

async fn write_instance_to_deployment_dir(
    instance: &ConfigInstance,
    deployment_dir: &Dir,
) -> Result<(), DeployErr> {

    // only write the config instance to the filesystem if it has a filepath
    let filepath = match &instance.filepath {
        Some(filepath) => filepath,
        None => return Ok(()),
    };

    let dest_file = deployment_dir.file(filepath);
    dest_file.write_json(&instance.instance, true, true).await.
        map_err(|e| DeployErr::DeployFileSysErr(DeployFileSysErr {
        source: e,
        trace: trace!(),
    }))
}

// =================================== REMOVE ====================================== //
pub async fn remove<O: Observer>(
    mut config_instance: ConfigInstance,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observer: &O,
) -> (ConfigInstance, Result<(), DeployErr>) {

    let result = delete_instance_from_deployment_dir(
        &config_instance, deployment_dir,
    ).await;

    match result {
        Ok(_) => {
            info!("Removed config instance '{}' from filesystem", config_instance.id);
            fsm::remove(&mut config_instance);
            if let Err(e) = observer.on_update(&config_instance) {
                return (config_instance, Err(e));
            }
            (config_instance, Ok(()))
        }
        Err(e) => {
            error!("Error removing config instance '{}': {:?}", config_instance.id, e);
            fsm::error(&mut config_instance, settings, &e);
            if let Err(e) = observer.on_update(&config_instance) {
                return (config_instance, Err(e));
            }
            (config_instance, Err(e))
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
    dest_file.delete().await.map_err(|e| DeployErr::DeployFileSysErr(DeployFileSysErr {
        source: e,
        trace: trace!(),
    }))
}