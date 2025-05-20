// standard crates
use std::fmt::Display;

// internal crates
use crate::storage::cache::Cache;
use crate::utils::PATH_DELIMITER;

// external crates
use serde::Deserialize;
use serde::Serialize;

pub type ConfigInstanceCache = Cache<ConfigInstanceCacheKey, ConfigInstance>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConfigInstanceCacheKey {
    pub config_slug: String,
    pub config_schema_digest: String,
}

impl Display for ConfigInstanceCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.config_slug, PATH_DELIMITER, self.config_schema_digest
        )
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigInstance {
    #[serde(rename = "config_instance_id")]
    pub id: String,
    pub created_at: String,
    pub client_id: String,
    pub config_schema_id: String,
    pub config_instance: serde_json::Value,

    // agent specific fields
    pub config_slug: String,
    pub config_schema_digest: String,
}
