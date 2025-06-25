// standard crates
use std::fmt::Display;

// internal crates
use crate::models::config_instance::ConfigInstance;
use crate::storage::cache::Cache;

// external crates
use serde::Deserialize;
use serde::Serialize;

// config instance cache
pub type ConfigInstanceCache = Cache<ConfigInstanceCacheKey, ConfigInstance>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConfigInstanceCacheKey {
    pub config_instance_id: String,
}

impl Display for ConfigInstanceCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.config_instance_id)
    }
}

// config instance data cache
pub type ConfigInstanceDataCache = Cache<ConfigInstanceCacheKey, serde_json::Value>;