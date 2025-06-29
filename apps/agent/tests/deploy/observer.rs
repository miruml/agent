// internal crates
use config_agent::models::config_instance::ConfigInstance;
use config_agent::deploy::errors::DeployErr;
use config_agent::deploy::observer::Observer;

// external crates
use async_trait::async_trait;

#[derive(Debug, Default)]
pub struct HistoryObserver {
    pub history: Vec<ConfigInstance>,
}

impl HistoryObserver {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }
}

#[async_trait]
impl Observer for HistoryObserver {
    async fn on_update(&mut self, instance: &ConfigInstance) -> Result<(), DeployErr> {
        self.history.push(instance.clone());
        Ok(())
    }
}
