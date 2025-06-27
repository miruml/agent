// internal crates
use crate::deploy::errors::{DeployErr, DeployStorageErr};
use crate::models::config_instance::ConfigInstance;
use crate::storage::config_instances::ConfigInstanceCache;
use crate::trace;

// external crates
use async_trait::async_trait;

#[async_trait]
pub trait Observer {
    async fn on_update(&mut self, config_instance: &ConfigInstance) -> Result<(), DeployErr>;
}

pub async fn on_update(
    observers: &mut [&mut dyn Observer],
    config_instance: &ConfigInstance,
) -> Result<(), DeployErr> {
    for observer in observers.iter_mut() {
        if let Err(e) = observer.on_update(config_instance).await {
            return Err(e);
        }
    }
    Ok(())
}

pub struct StorageObserver<'a> {
    pub cfg_inst_cache: &'a ConfigInstanceCache,
}

#[async_trait]
impl<'a> Observer for StorageObserver<'a> {
    async fn on_update(&mut self, config_instance: &ConfigInstance) -> Result<(), DeployErr> {
        self.cfg_inst_cache.write(
            config_instance.id.clone(),
            config_instance.clone(),
            true,
        ).await.map_err(|e| {
            DeployErr::StorageErr(DeployStorageErr {
                source: e,
                trace: trace!(),
            })
        })
    }
}
