// internal crates
use crate::storage::cache::Cache;

// external crates
use serde::Deserialize;
use serde::Serialize;

pub type RawSchemaDigest = String;
pub type ConfigSchemaDigestCache = Cache<RawSchemaDigest, ConfigSchemaDigests>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConfigSchemaDigests {
    pub raw: String,
    pub resolved: String,
}
