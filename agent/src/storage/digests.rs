// internal crates
use crate::cache::{entry::CacheEntry, file::FileCache};

// external crates
use serde::Deserialize;
use serde::Serialize;

pub type RawSchemaDigest = String;
pub type ConfigSchemaDigestCacheEntry = CacheEntry<RawSchemaDigest, ConfigSchemaDigests>;
pub type ConfigSchemaDigestCache = FileCache<RawSchemaDigest, ConfigSchemaDigests>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ConfigSchemaDigests {
    pub raw: String,
    pub resolved: String,
}
