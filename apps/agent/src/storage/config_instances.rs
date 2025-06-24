// standard crates
use std::fmt::Display;

// internal crates
use crate::models::config_instance::ConfigInstance;
use crate::storage::cache::Cache;
use crate::utils::PATH_DELIMITER;

// external crates
use serde::Deserialize;
use serde::Serialize;

pub type ConfigInstanceCache = Cache<ConfigInstanceCacheKey, ConfigInstance>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConfigInstanceCacheKey {
    pub config_type_slug: String,
    pub config_schema_digest: String,
}

impl Display for ConfigInstanceCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.config_type_slug, PATH_DELIMITER, self.config_schema_digest
        )
    }
}
