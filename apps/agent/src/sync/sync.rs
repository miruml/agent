use crate::crud::prelude::*;
use crate::deploy::{
    apply::apply_deployments,
    errors::{DeployErr, DeployStorageErr},
    fsm,
    observer::Observer,
};
use crate::filesys::dir::Dir;
use crate::http::config_instances::ConfigInstancesExt;
use crate::models::config_instance::ConfigInstance;
use crate::storage::config_instances::{
    ConfigInstanceCache,
    ConfigInstanceCacheEntry,
    ConfigInstanceDataCache,
};
use crate::sync::pull::pull_config_instances;
use crate::sync::push::push_config_instances;
use crate::sync::errors::{
    SyncErr,
    SyncCrudErr,
    SyncDeployErr,
};
use crate::trace;

// external crates
use async_trait::async_trait;

pub async fn sync_config_instances<HTTPClientT: ConfigInstancesExt>(
    cfg_inst_cache: &ConfigInstanceCache,
    cfg_inst_data_cache: &ConfigInstanceDataCache,
    http_client: &HTTPClientT,
    device_id: &str,
    token: &str,
    deployment_dir: &Dir,
    fsm_settings: &fsm::Settings,
) -> Result<(), SyncErr> {

    // pull config instances from server
    pull_config_instances(
        cfg_inst_cache,
        cfg_inst_data_cache,
        http_client,
        device_id,
        token,
    ).await?;

    // read the config instances which need to be applied
    let cfg_insts_to_apply = cfg_inst_cache.find_all(
        |instance| { fsm::is_action_required(instance) }
    ).await.map_err(|e| SyncErr::CrudErr(SyncCrudErr {
        source: e,
        trace: trace!(),
    }))?;
    let cfg_insts_to_apply = cfg_insts_to_apply.into_iter().map(|instance| (instance.id.clone(), instance)).collect();

    // observers
    let mut observers: Vec<&mut dyn Observer> = Vec::new();
    let mut storage_observer = StorageObserver { cfg_inst_cache };
    observers.push(&mut storage_observer);

    // apply deployments
    apply_deployments(
        cfg_insts_to_apply,
        cfg_inst_cache,
        cfg_inst_data_cache,
        deployment_dir,
        fsm_settings,
        &mut observers[..],
    ).await.map_err(|e| SyncErr::DeployErr(SyncDeployErr {
        source: e,
        trace: trace!(),
    }))?;

    // push config instances to server
    push_config_instances(
        cfg_inst_cache,
        http_client,
        token,
    ).await?;

    Ok(())
}


pub struct StorageObserver<'a> {
    pub cfg_inst_cache: &'a ConfigInstanceCache,
}

#[async_trait]
impl<'a> Observer for StorageObserver<'a> {
    async fn on_update(&mut self, instance: &ConfigInstance) -> Result<(), DeployErr> {
        let overwrite = true;
        self.cfg_inst_cache.write(
            instance.id.clone(),
            instance.clone(),
            is_dirty,
            overwrite,
        ).await.map_err(|e| {
            DeployErr::StorageErr(DeployStorageErr {
                source: e,
                trace: trace!(),
            })
        })
    }
}

fn is_dirty(old: Option<&ConfigInstanceCacheEntry>, new: &ConfigInstance) -> bool {
    let old = match old {
        Some(old) => old,
        None => return true,
    };
    old.is_dirty ||
    old.value.activity_status != new.activity_status || 
    old.value.error_status != new.error_status
}