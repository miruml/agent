// internal crates
use crate::crud::prelude::Read;
use crate::deploy::errors::{DeployCrudErr, DeployErr, DeployFileSysErr};
use crate::deploy::fsm;
use crate::deploy::observer::{on_update, Observer};
use crate::filesys::dir::Dir;
use crate::models::config_instance::{ConfigInstance, ConfigInstanceID, TargetStatus};
use crate::trace;

// external crates
use tracing::{error, info};

#[derive(Debug)]
pub struct DeployResults {
    pub to_remove: Vec<ConfigInstance>,
    pub to_deploy: Vec<ConfigInstance>,
}

impl DeployResults {
    pub fn empty() -> Self {
        DeployResults {
            to_remove: Vec::new(),
            to_deploy: Vec::new(),
        }
    }
}

pub async fn deploy_with_rollback<R>(
    to_remove: Vec<ConfigInstance>,
    to_deploy: Vec<ConfigInstance>,
    cfg_inst_content_reader: &R,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (DeployResults, Result<(), DeployErr>)
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    // remove the previous config instance. Don't worry whether it failed or not as we want
    // to attempt to deploy the next config instance regardless
    let (to_remove, result) = remove_many(to_remove, deployment_dir, settings, observers).await;
    if let Err(e) = result {
        error!("Error removing config instances: {:?}", e);
    }

    // deploy the new configinstance
    let (to_deploy, result) = deploy_many(
        to_deploy,
        cfg_inst_content_reader,
        deployment_dir,
        settings,
        observers,
    )
    .await;
    if let Err(e) = result {
        error!("Error deploying config instances: {:?}", e);
    } else {
        return (
            DeployResults {
                to_remove,
                to_deploy,
            },
            Ok(()),
        );
    }

    // deployment FAILED -> rollback to the previous configinstance

    // remove the attempted deployment. Don't worry whether it failed or not as there
    // is nothing to do at this point. It will be attempted again with a retry.
    let (to_deploy, result) = remove_many(to_deploy, deployment_dir, settings, observers).await;
    if let Err(e) = result {
        error!("Error stopping deployment: {:?}", e);
    }

    // deploy the previous instance. Don't worry whether it failed or not as there
    // is nothing to do at this point. It will be attempted again with a retry next
    // time.
    let (to_remove, result) = deploy_many(
        to_remove,
        cfg_inst_content_reader,
        deployment_dir,
        settings,
        observers,
    )
    .await;
    if let Err(e) = result {
        error!("Error booting deployment: {:?}", e);
    }

    (
        DeployResults {
            to_remove,
            to_deploy,
        },
        Ok(()),
    )
}

// =================================== DEPLOY ====================================== //
async fn deploy_many<R>(
    cfg_insts: Vec<ConfigInstance>,
    content_fetcher: &R,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (Vec<ConfigInstance>, Result<(), DeployErr>)
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    let mut post_deploy_cfg_insts = Vec::new();
    let mut cfg_insts_iter = cfg_insts.into_iter();
    while let Some(cfg_inst) = cfg_insts_iter.next() {
        let (post_deploy_cfg_inst, result) = deploy(
            cfg_inst,
            content_fetcher,
            deployment_dir,
            settings,
            observers,
        )
        .await;
        if let Err(e) = result {
            // add the current post_deploy_cfg_inst
            post_deploy_cfg_insts.push(post_deploy_cfg_inst);
            // add the rest of the unprocessed config instances
            for remaining_cfg_inst in cfg_insts_iter {
                post_deploy_cfg_insts.push(remaining_cfg_inst);
            }
            return (post_deploy_cfg_insts, Err(e));
        }
        post_deploy_cfg_insts.push(post_deploy_cfg_inst);
    }
    (post_deploy_cfg_insts, Ok(()))
}

async fn deploy<R>(
    mut cfg_inst: ConfigInstance,
    content_fetcher: &R,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (ConfigInstance, Result<(), DeployErr>)
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    let result = write_cfg_inst_to_deployment_dir(&cfg_inst, content_fetcher, deployment_dir).await;

    match result {
        Ok(_) => {
            info!("Deployed config instance '{}' to filesystem", cfg_inst.id);
            cfg_inst = fsm::deploy(cfg_inst);
            if let Err(e) = on_update(observers, &cfg_inst).await {
                return (cfg_inst, Err(e));
            }
            (cfg_inst, Ok(()))
        }
        Err(e) => {
            let increment_attempts = cfg_inst.target_status == TargetStatus::Deployed;
            cfg_inst = fsm::error(cfg_inst, settings, &e, increment_attempts);
            if let Err(e) = on_update(observers, &cfg_inst).await {
                return (cfg_inst, Err(e));
            }
            (cfg_inst, Err(e))
        }
    }
}

async fn write_cfg_inst_to_deployment_dir<R>(
    cfg_inst: &ConfigInstance,
    content_fetcher: &R,
    deployment_dir: &Dir,
) -> Result<(), DeployErr>
where
    R: Read<ConfigInstanceID, serde_json::Value>,
{
    // only write the config instance to the filesystem if it has a filepath
    let rel_filepath = match &cfg_inst.relative_filepath {
        Some(filepath) => filepath,
        None => return Ok(()),
    };

    let cfg_inst_content = content_fetcher
        .read(cfg_inst.id.clone())
        .await
        .map_err(|e| {
            DeployErr::CrudErr(Box::new(DeployCrudErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    let dest_file = deployment_dir.file(rel_filepath);
    dest_file
        .write_json(&cfg_inst_content, true, true)
        .await
        .map_err(|e| {
            DeployErr::FileSysErr(Box::new(DeployFileSysErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    prune_deployment_dir(deployment_dir).await;

    Ok(())
}

// =================================== REMOVE ====================================== //
async fn remove_many(
    cfg_insts: Vec<ConfigInstance>,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (Vec<ConfigInstance>, Result<(), DeployErr>) {
    let mut post_remove_cfg_insts = Vec::new();
    let mut cfg_insts_iter = cfg_insts.into_iter();
    while let Some(cfg_inst) = cfg_insts_iter.next() {
        let (post_remove_cfg_inst, result) =
            remove(cfg_inst, deployment_dir, settings, observers).await;
        if let Err(e) = result {
            // add the current post_remove_cfg_inst
            post_remove_cfg_insts.push(post_remove_cfg_inst);
            // add the rest of the unprocessed config instances
            for remaining_cfg_inst in cfg_insts_iter {
                post_remove_cfg_insts.push(remaining_cfg_inst);
            }
            return (post_remove_cfg_insts, Err(e));
        }
        post_remove_cfg_insts.push(post_remove_cfg_inst);
    }
    (post_remove_cfg_insts, Ok(()))
}

async fn remove(
    mut cfg_inst: ConfigInstance,
    deployment_dir: &Dir,
    settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (ConfigInstance, Result<(), DeployErr>) {
    let result = delete_cfg_inst_from_deployment_dir(&cfg_inst, deployment_dir).await;

    match result {
        Ok(_) => {
            info!("Removed config instance '{}' from filesystem", cfg_inst.id);
            cfg_inst = fsm::remove(cfg_inst);
            if let Err(e) = on_update(observers, &cfg_inst).await {
                return (cfg_inst, Err(e));
            }
            (cfg_inst, Ok(()))
        }
        Err(e) => {
            let increment_attempts = cfg_inst.target_status == TargetStatus::Removed;
            cfg_inst = fsm::error(cfg_inst, settings, &e, increment_attempts);
            if let Err(e) = on_update(observers, &cfg_inst).await {
                return (cfg_inst, Err(e));
            }
            (cfg_inst, Err(e))
        }
    }
}

async fn delete_cfg_inst_from_deployment_dir(
    cfg_inst: &ConfigInstance,
    deployment_dir: &Dir,
) -> Result<(), DeployErr> {
    // only delete the configs from the filesystem if it has a filepath
    let rel_filepath = match &cfg_inst.relative_filepath {
        Some(filepath) => filepath,
        None => return Ok(()),
    };

    let dest_file = deployment_dir.file(rel_filepath);
    dest_file.delete().await.map_err(|e| {
        DeployErr::FileSysErr(Box::new(DeployFileSysErr {
            source: e,
            trace: trace!(),
        }))
    })?;

    prune_deployment_dir(deployment_dir).await;

    Ok(())
}

async fn prune_deployment_dir(deployment_dir: &Dir) {
    let subdirs = match deployment_dir.subdirs().await {
        Ok(subdirs) => subdirs,
        Err(e) => {
            error!("Error determining deployment subdirs for pruning: {:?}", e);
            return;
        }
    };
    for subdir in subdirs {
        if let Err(e) = subdir.delete_if_empty_recursive().await {
            error!(
                "Error pruning deployment subdir directory {:?}: {:?}",
                subdir, e
            );
        } else {
            info!("Pruned deployment subdir directory {:?}", subdir);
        }
    }
}
