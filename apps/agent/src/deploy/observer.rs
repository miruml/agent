// internal crates
use crate::deploy::errors::DeployErr;
use crate::models::config_instance::ConfigInstance;

// external crates
use async_trait::async_trait;

#[async_trait]
pub trait Observer: Send {
    async fn on_update(&mut self, config_instance: &ConfigInstance) -> Result<(), DeployErr>;
}

pub async fn on_update(
    observers: &mut [&mut dyn Observer],
    config_instance: &ConfigInstance,
) -> Result<(), DeployErr> {
    for observer in observers.iter_mut() {
        observer.on_update(config_instance).await?
    }
    Ok(())
}