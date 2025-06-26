// internal crates
use crate::models::config_instance::ConfigInstance;
use crate::storage::cache::Cache;

// config instance cache
pub type ConfigInstanceID = String;
pub type ConfigInstanceCache = Cache<ConfigInstanceID, ConfigInstance>;
pub type ConfigInstanceDataCache = Cache<ConfigInstanceID, serde_json::Value>;