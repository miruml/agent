// standard crates
use std::collections::HashMap;

// internal crates
use crate::crud::config_instance::{
    matches_config_schema_and_activity_status, matches_filepath_and_activity_status,
};
use crate::crud::prelude::{Find, Read};
use crate::deploy::errors::{
    ConflictingDeploymentsErr, DeployCacheErr, DeployCrudErr, DeployErr, InstanceNotDeployableErr,
};
use crate::deploy::{
    filesys,
    filesys::DeployResults,
    fsm,
    observer::{on_update, Observer},
};
use crate::filesys::dir::Dir;
use crate::models::config_instance::{
    ActivityStatus, ConfigInstance, ConfigInstanceID, TargetStatus,
};
use crate::storage::config_instances::{
    ConfigInstanceCache, ConfigInstanceCacheEntry, ConfigInstanceDataCache,
};
use crate::trace;

// external crates
use async_trait::async_trait;
use tracing::{error, info};

pub fn is_dirty(old: Option<&ConfigInstanceCacheEntry>, new: &ConfigInstance) -> bool {
    let old = match old {
        Some(old) => old,
        None => return true,
    };
    old.is_dirty
        || old.value.activity_status != new.activity_status
        || old.value.error_status != new.error_status
}

pub struct StorageObserver<'a> {
    pub cfg_inst_cache: &'a ConfigInstanceCache,
}

#[async_trait]
impl<'a> Observer for StorageObserver<'a> {
    async fn on_update(&mut self, instance: &ConfigInstance) -> Result<(), DeployErr> {
        let overwrite = true;
        self.cfg_inst_cache
            .write(instance.id.clone(), instance.clone(), is_dirty, overwrite)
            .await
            .map_err(|e| {
                DeployErr::CacheErr(Box::new(DeployCacheErr {
                    source: e,
                    trace: trace!(),
                }))
            })
    }
}

pub async fn apply(
    mut cfg_insts_to_apply: HashMap<ConfigInstanceID, ConfigInstance>,
    cfg_inst_cache: &ConfigInstanceCache,
    cfg_inst_data_cache: &ConfigInstanceDataCache,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
) -> Result<HashMap<ConfigInstanceID, ConfigInstance>, DeployErr> {
    // observers
    let mut observers: Vec<&mut dyn Observer> = Vec::new();
    let mut storage_observer = StorageObserver { cfg_inst_cache };
    observers.push(&mut storage_observer);

    let mut applied_cfg_insts = HashMap::new();

    // apply the deployments until there are none left to apply
    let max_iters = 30;
    let mut i = 0;
    while !cfg_insts_to_apply.is_empty() {
        i += 1;
        if i > max_iters {
            error!("Max iterations reached while applying deployments, exiting");
            break;
        }

        let id = match cfg_insts_to_apply.keys().next().cloned() {
            Some(id) => id,
            None => break,
        };
        let cfg_inst = match cfg_insts_to_apply.remove(&id) {
            Some(cfg_inst) => cfg_inst,
            None => break,
        };

        // apply the deployment
        let (instance_results, result) = apply_one(
            cfg_inst,
            cfg_inst_cache,
            cfg_inst_data_cache,
            deployment_dir,
            fsm_settings,
            &mut observers,
        )
        .await;
        if let Err(e) = result {
            error!("Error applying config instance {:?}: {:?}", id, e);
        }

        // update the config instances to apply
        for instance in instance_results.to_remove.into_iter() {
            if fsm::is_action_required(fsm::next_action(&instance, true)) {
                cfg_insts_to_apply.insert(instance.id.clone(), instance);
            } else {
                cfg_insts_to_apply.remove(&instance.id);
                applied_cfg_insts.insert(instance.id.clone(), instance);
            }
        }
        for instance in instance_results.to_deploy.into_iter() {
            if fsm::is_action_required(fsm::next_action(&instance, true)) {
                cfg_insts_to_apply.insert(instance.id.clone(), instance);
            } else {
                cfg_insts_to_apply.remove(&instance.id);
                applied_cfg_insts.insert(instance.id.clone(), instance);
            }
        }
    }

    Ok(applied_cfg_insts)
}

async fn apply_one<R1, R2>(
    cfg_inst: ConfigInstance,
    all_cfg_insts: &R1,
    all_cfg_insts_data: &R2,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (DeployResults, Result<(), DeployErr>)
where
    R1: Find<ConfigInstanceID, ConfigInstance>,
    R2: Read<ConfigInstanceID, serde_json::Value>,
{
    match fsm::next_action(&cfg_inst, true) {
        fsm::NextAction::None => (DeployResults::empty(), Ok(())),
        fsm::NextAction::Deploy => {
            deploy(
                cfg_inst,
                all_cfg_insts,
                all_cfg_insts_data,
                deployment_dir,
                fsm_settings,
                observers,
            )
            .await
        }
        fsm::NextAction::Remove => {
            remove(
                cfg_inst,
                all_cfg_insts,
                all_cfg_insts_data,
                deployment_dir,
                fsm_settings,
                observers,
            )
            .await
        }
        fsm::NextAction::Wait(_) => (DeployResults::empty(), Ok(())),
    }
}

async fn deploy<R1, R2>(
    cfg_inst: ConfigInstance,
    all_cfg_insts: &R1,
    all_cfg_insts_data: &R2,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (DeployResults, Result<(), DeployErr>)
where
    R1: Find<ConfigInstanceID, ConfigInstance>,
    R2: Read<ConfigInstanceID, serde_json::Value>,
{
    if fsm::next_action(&cfg_inst, true) != fsm::NextAction::Deploy {
        let next_action = fsm::next_action(&cfg_inst, true);
        return (
            DeployResults::empty(),
            Err(DeployErr::InstanceNotDeployableErr(Box::new(
                InstanceNotDeployableErr {
                    instance: cfg_inst,
                    next_action,
                    trace: trace!(),
                },
            ))),
        );
    }

    // find the conflicts to remove
    let conflicts = match find_instances_to_replace(&cfg_inst, all_cfg_insts).await {
        Ok(conflicts) => conflicts,
        Err(e) => return (DeployResults::empty(), Err(e)),
    };

    let replacement_ids = conflicts.iter().map(|c| &c.id).collect::<Vec<_>>();
    info!(
        "deploying config instance {:?} and removing {:?}",
        cfg_inst.id, replacement_ids
    );

    // remove the old instances and deploy the new instance
    filesys::deploy_with_rollback(
        conflicts,
        vec![cfg_inst],
        all_cfg_insts_data,
        deployment_dir,
        fsm_settings,
        observers,
    )
    .await
}

async fn remove<R1, R2>(
    mut cfg_inst: ConfigInstance,
    all_cfg_insts: &R1,
    all_cfg_insts_data: &R2,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (DeployResults, Result<(), DeployErr>)
where
    R1: Find<ConfigInstanceID, ConfigInstance>,
    R2: Read<ConfigInstanceID, serde_json::Value>,
{
    if fsm::next_action(&cfg_inst, true) != fsm::NextAction::Remove {
        let next_action = fsm::next_action(&cfg_inst, true);
        return (
            DeployResults::empty(),
            Err(DeployErr::InstanceNotDeployableErr(Box::new(
                InstanceNotDeployableErr {
                    instance: cfg_inst,
                    next_action,
                    trace: trace!(),
                },
            ))),
        );
    }

    // find the replacements to deploy
    let replacement = match find_replacement(&cfg_inst, all_cfg_insts).await {
        Ok(replacement) => replacement,
        Err(e) => return (DeployResults::empty(), Err(e)),
    };

    let mut replacements = vec![];
    if let Some(replacement) = replacement {
        // if a replacement exists and is in cooldown, we must wait for the new instance
        // to finish cooling down before we can remove this one. Thus, this instance
        // receives the same cooldown as the replacement so that they are removed at the
        // same time.
        if replacement.is_in_cooldown() {
            cfg_inst.set_cooldown(replacement.cooldown());
            let err_results = on_update(observers, &cfg_inst).await;
            let deploy_results = DeployResults {
                to_remove: vec![],
                to_deploy: vec![cfg_inst],
            };
            return (deploy_results, err_results);
        }
        replacements.push(replacement);
    }

    let replacement_ids = replacements.iter().map(|r| &r.id).collect::<Vec<_>>();
    info!(
        "removing config instance {:?} and replacing it with {:?}",
        cfg_inst.id, replacement_ids
    );

    // remove the instance and deploy the replacement
    filesys::deploy_with_rollback(
        vec![cfg_inst],
        replacements,
        all_cfg_insts_data,
        deployment_dir,
        fsm_settings,
        observers,
    )
    .await
}

pub async fn find_instances_to_replace<R>(
    cfg_inst: &ConfigInstance,
    all_cfg_insts: &R,
) -> Result<Vec<ConfigInstance>, DeployErr>
where
    R: Find<ConfigInstanceID, ConfigInstance>,
{
    let opt_filepath = cfg_inst.relative_filepath.clone();
    let cfg_sch_id = cfg_inst.config_schema_id.clone();
    let conflicts = all_cfg_insts
        .find_where(move |cfg_inst| {
            // is deployed and has same config schema
            if matches_config_schema_and_activity_status(
                cfg_inst,
                &cfg_sch_id,
                ActivityStatus::Deployed,
            ) {
                return true;
            }

            // is deployed and has same filepath
            let filepath = match opt_filepath.as_ref() {
                Some(filepath) => filepath,
                None => return false,
            };
            matches_filepath_and_activity_status(cfg_inst, filepath, ActivityStatus::Deployed)
        })
        .await
        .map_err(|e| {
            DeployErr::CrudErr(Box::new(DeployCrudErr {
                source: e,
                trace: trace!(),
            }))
        })?;

    // validate that all conflicts do not desire to be deployed
    for conflict in conflicts.iter() {
        if conflict.target_status == TargetStatus::Deployed {
            return Err(DeployErr::ConflictingDeploymentsErr(Box::new(
                ConflictingDeploymentsErr {
                    instances: vec![cfg_inst.clone(), conflict.clone()],
                    trace: trace!(),
                },
            )));
        }
    }

    Ok(conflicts)
}

pub async fn find_replacement<R>(
    cfg_inst: &ConfigInstance,
    all_cfg_insts: &R,
) -> Result<Option<ConfigInstance>, DeployErr>
where
    R: Find<ConfigInstanceID, ConfigInstance>,
{
    let cfg_sch_id = cfg_inst.config_schema_id.clone();
    all_cfg_insts
        .find_one_optional("filter by config schema and next action", move |cfg_inst| {
            // has same config schema and desires to be deployed
            matches_config_schema_and_next_action(
                cfg_inst,
                &cfg_sch_id,
                fsm::NextAction::Deploy,
                false,
            )
        })
        .await
        .map_err(|e| {
            DeployErr::CrudErr(Box::new(DeployCrudErr {
                source: e,
                trace: trace!(),
            }))
        })
}

pub fn matches_config_schema_and_next_action(
    instance: &ConfigInstance,
    config_schema_id: &str,
    next_action: fsm::NextAction,
    use_cooldown: bool,
) -> bool {
    instance.config_schema_id == config_schema_id
        && fsm::next_action(instance, use_cooldown) == next_action
}
