// standard crates
use std::fmt::Display;

// internal crates
use crate::storage::cache::Cache;
use crate::utils::PATH_DELIMITER;

// external crates
use serde::Deserialize;
use serde::Serialize;

pub type ConcreteConfigCache = Cache<ConcreteConfigCacheKey, ConcreteConfig>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConcreteConfigCacheKey {
    pub config_slug: String,
    pub config_schema_digest: String,
}

impl Display for ConcreteConfigCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.config_slug, PATH_DELIMITER, self.config_schema_digest
        )
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConcreteConfig {
    #[serde(rename = "concrete_config_id")]
    pub id: String,
    pub created_at: String,
    pub client_id: String,
    pub config_schema_id: String,
    pub concrete_config: serde_json::Value,

    // agent specific fields
    pub config_slug: String,
    pub config_schema_digest: String,
}
