// internal crates
use crate::models::config_instance::{
    ConfigInstanceID,
    ConfigInstance,
};
use crate::storage::cache::{Cache, CacheEntry};

// config instance cache
pub type ConfigInstanceCacheEntry = CacheEntry<ConfigInstanceID, ConfigInstance>;
pub type ConfigInstanceCache = Cache<ConfigInstanceID, ConfigInstance>;
pub type ConfigInstanceDataCache = Cache<ConfigInstanceID, serde_json::Value>;