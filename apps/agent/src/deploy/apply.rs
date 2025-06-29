// standard crates
use std::collections::HashMap;

// internal crates
use crate::crud::config_instance::{
    matches_config_schema_and_activity_status,
    matches_filepath_and_activity_status,
};
use crate::crud::prelude::{Find, Read};
use crate::deploy::errors::{
    DeployErr, DeployCrudErr, ConflictingDeploymentsErr, InstanceNotDeployableErr,
};
use crate::deploy::filesys;
use crate::deploy::fsm;
use crate::deploy::observer::Observer;
use crate::filesys::dir::Dir;
use crate::models::config_instance::{
    ConfigInstanceID,
    ConfigInstance,
    ActivityStatus,
    TargetStatus,
};
use crate::trace;

// external crates
use tracing::error;


struct InstanceResults {
    removed: Vec<ConfigInstance>,
    deployed: Vec<ConfigInstance>,
}

fn empty_results() -> InstanceResults {
    InstanceResults {
        removed: Vec::new(),
        deployed: Vec::new(),
    }
}

pub async fn apply_deployments<R1, R2>(
    mut cfg_insts_to_apply: HashMap<ConfigInstanceID, ConfigInstance>,
    all_cfg_insts: &R1,
    all_cfg_insts_data: &R2,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> Result<HashMap<ConfigInstanceID, ConfigInstance>, DeployErr>
where
    R1: Find<ConfigInstanceID, ConfigInstance>,
    R2: Read<ConfigInstanceID, serde_json::Value>,
{
    let mut applied_cfg_insts = HashMap::new();

    // apply the deployments until there are none left to apply
    while !cfg_insts_to_apply.is_empty() {
        let id = match cfg_insts_to_apply.keys().next().cloned() {
            Some(id) => id,
            None => break,
        };
        let cfg_inst = match cfg_insts_to_apply.remove(&id) {
            Some(cfg_inst) => cfg_inst,
            None => break,
        };

        // apply the deployment
        let (instance_results, result) = apply_deployment(
            cfg_inst,
            all_cfg_insts,
            all_cfg_insts_data,
            deployment_dir,
            fsm_settings,
            observers,
        ).await;
        if let Err(e) = result {
            error!("Error applying config instance {:?}: {:?}", id, e);
        }

        // update the config instances to apply
        for removal in instance_results.removed.into_iter() {
            if fsm::is_action_required(fsm::next_action(&removal, true)) {
                cfg_insts_to_apply.insert(removal.id.clone(), removal);
            } else {
                applied_cfg_insts.insert(removal.id.clone(), removal);
            }
        }
        for deployment in instance_results.deployed.into_iter() {
            if fsm::is_action_required(fsm::next_action(&deployment, true)) {
                cfg_insts_to_apply.insert(deployment.id.clone(), deployment);
            } else {
                applied_cfg_insts.insert(deployment.id.clone(), deployment);
            }
        }
    }

    Ok(applied_cfg_insts)
}



async fn apply_deployment<R1,R2>(
    cfg_inst: ConfigInstance,
    all_cfg_insts: &R1,
    all_cfg_insts_data: &R2,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (InstanceResults, Result<(), DeployErr>)
where
    R1: Find<ConfigInstanceID, ConfigInstance>,
    R2: Read<ConfigInstanceID, serde_json::Value>,
{
    match fsm::next_action(&cfg_inst, true) {
        fsm::NextAction::None => {
            (empty_results(), Ok(()))
        }
        fsm::NextAction::Deploy => {
            deploy(
                cfg_inst,
                all_cfg_insts,
                all_cfg_insts_data,
                deployment_dir,
                fsm_settings,
                observers,
            ).await
        }
        fsm::NextAction::Remove => {
            remove(
                cfg_inst,
                all_cfg_insts,
                all_cfg_insts_data,
                deployment_dir,
                fsm_settings,
                observers,
            ).await
        }
        fsm::NextAction::Wait(_) => {
            (empty_results(), Ok(()))
        }
    }
}

async fn deploy<R1, R2>(
    cfg_inst: ConfigInstance,
    all_cfg_insts: &R1,
    all_cfg_insts_data: &R2,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (InstanceResults, Result<(), DeployErr>)
where
    R1: Find<ConfigInstanceID, ConfigInstance>,
    R2: Read<ConfigInstanceID, serde_json::Value>,
{
    if fsm::next_action(&cfg_inst, true) != fsm::NextAction::Deploy {
        let next_action = fsm::next_action(&cfg_inst, true);
        return (empty_results(), Err(DeployErr::InstanceNotDeployableErr(InstanceNotDeployableErr {
            instance: cfg_inst,
            next_action,
            trace: trace!(),
        })));
    }

    // find the conflicts to remove
    let conflicts= match find_instances_to_replace(
        &cfg_inst, all_cfg_insts,
    ).await {
        Ok(conflicts) => conflicts,
        Err(e) => return (empty_results(), Err(e)),
    };

    // remove the old instances and deploy the new instance
    let (removed_instances, deployed_instances, result) = filesys::deploy_with_rollback(
        conflicts,
        vec![cfg_inst],
        all_cfg_insts_data,
        deployment_dir,
        fsm_settings,
        observers,
    ).await;
    let instance_results = InstanceResults {
        removed: removed_instances,
        deployed: deployed_instances,
    };

    match result {
        Ok(_) => (instance_results, Ok(())),
        Err(e) => (instance_results, Err(e)),
    }
}

async fn remove<R1, R2>(
    cfg_inst: ConfigInstance,
    all_cfg_insts: &R1,
    all_cfg_insts_data: &R2,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
    observers: &mut [&mut dyn Observer],
) -> (InstanceResults, Result<(), DeployErr>)
where
    R1: Find<ConfigInstanceID, ConfigInstance>,
    R2: Read<ConfigInstanceID, serde_json::Value>,
{

    if fsm::next_action(&cfg_inst, true) != fsm::NextAction::Remove {
        let next_action = fsm::next_action(&cfg_inst, true);
        return (empty_results(), Err(DeployErr::InstanceNotDeployableErr(InstanceNotDeployableErr {
            instance: cfg_inst,
            next_action,
            trace: trace!(),
        })));
    }

    // find the replacements to deploy
    let replacement = match find_replacement(&cfg_inst, all_cfg_insts).await {
        Ok(replacement) => replacement,
        Err(e) => return (empty_results(), Err(e)),
    };
    let mut replacements = vec![];
    if let Some(replacement) = replacement {
        replacements.push(replacement);
    }

    // remove the instance and deploy the replacement
    let (removed_instances, deployed_instances, result) = filesys::deploy_with_rollback(
        vec![cfg_inst],
        replacements,
        all_cfg_insts_data,
        deployment_dir,
        fsm_settings,
        observers,
    ).await;
    let instance_results = InstanceResults {
        removed: removed_instances,
        deployed: deployed_instances,
    };

    match result {
        Ok(_) => (instance_results, Ok(())),
        Err(e) => (instance_results, Err(e)),
    }
}

async fn find_instances_to_replace<R>(
    cfg_inst: &ConfigInstance,
    all_cfg_insts: &R,
) -> Result<Vec<ConfigInstance>, DeployErr>
where
    R: Find<ConfigInstanceID, ConfigInstance>,
{

    let opt_filepath= cfg_inst.filepath.clone();
    let cfg_sch_id = cfg_inst.config_schema_id.clone();
    let conflicts = all_cfg_insts.find_where(
        move |cfg_inst| {
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
            matches_filepath_and_activity_status(
                cfg_inst, filepath, ActivityStatus::Deployed,
            )
        },
    ).await.map_err(|e| {
        DeployErr::CrudErr(DeployCrudErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // validate that all conflicts do not desire to be deployed
    for conflict in conflicts.iter() {
        if conflict.target_status == TargetStatus::Deployed {
            return Err(DeployErr::ConflictingDeploymentsErr(ConflictingDeploymentsErr {
                instances: vec![cfg_inst.clone(), conflict.clone()],
                trace: trace!(),
            }));
        }
    }

    Ok(conflicts)
}

async fn find_replacement<R>(
    cfg_inst: &ConfigInstance,
    all_cfg_insts: &R,
) -> Result<Option<ConfigInstance>, DeployErr>
where
    R: Find<ConfigInstanceID, ConfigInstance>,
{
    let cfg_sch_id = cfg_inst.config_schema_id.clone();
    all_cfg_insts.find_one_optional(
        "filter by config schema and next action",
        move |cfg_inst| {
            // has same config schema and desires to be deployed 
            matches_config_schema_and_next_action(
                cfg_inst, &cfg_sch_id,
                fsm::NextAction::Deploy, false,
            )
        },
    ).await.map_err(|e| {
        DeployErr::CrudErr(DeployCrudErr {
            source: e,
            trace: trace!(),
        })
    })
}

pub fn matches_config_schema_and_next_action(
    instance: &ConfigInstance,
    config_schema_id: &str,
    next_action: fsm::NextAction,
    use_cooldown: bool,
) -> bool {
    instance.config_schema_id == config_schema_id &&
    fsm::next_action(instance, use_cooldown) == next_action
}