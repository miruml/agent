// internal crates
use crate::models::config_instance::{ConfigInstance, ConfigInstanceActivityStatus};
use crate::storage::cache::{Cache, CacheEntry};

// config instance cache
pub type ConfigInstanceID = String;
pub type ConfigInstanceCache = Cache<ConfigInstanceID, ConfigInstance>;
pub type ConfigInstanceDataCache = Cache<ConfigInstanceID, serde_json::Value>;

// queries
pub fn filter_by_config_schema_and_activity_status(
    entry: &CacheEntry<ConfigInstanceID, ConfigInstance>,
    config_schema_id: &str,
    activity_status: ConfigInstanceActivityStatus,
) -> bool {
    entry.value.config_schema_id == config_schema_id && entry.value.activity_status == activity_status
}

pub fn filter_by_filepath_and_activity_status(
    entry: &CacheEntry<ConfigInstanceID, ConfigInstance>,
    filepath: &str,
    status: ConfigInstanceActivityStatus,
) -> bool {
    let entry_filepath = match &entry.value.filepath {
        Some(filepath) => filepath,
        None => return false,
    };
    filepath == entry_filepath && status == entry.value.activity_status
}
