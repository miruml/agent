// internal crates
use crate::storage::cache::Cache;
use crate::utils::PATH_DELIMITER;

// external crates
use serde::Deserialize;
use serde::Serialize;

pub type ConcreteConfigCache = Cache<ConcreteConfigCacheKey, ConcreteConfig>;

pub struct ConcreteConfigCacheKey {
    pub config_slug: String,
    pub config_schema_digest: String,
}

impl ToString for ConcreteConfigCacheKey {
    fn to_string(&self) -> String {
        format!("{}{}{}", self.config_slug, PATH_DELIMITER, self.config_schema_digest)
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
