// internal crates
use crate::cache::{dir::DirCache, entry::CacheEntry, file::FileCache};
use crate::models::config_instance::{ConfigInstance, ConfigInstanceID};

pub type ConfigInstanceCacheEntry = CacheEntry<ConfigInstanceID, ConfigInstance>;

// the config instance metadata cache is all stored in a single file while the config
// instance data is stored in a directory with a file for each cache entry. This is
// for performance reasons since we primarily access the metadata and storing it in a
// single file allows for better performance / caching by the OS. On the other hand,
// the actual configuration data can be quite large so they each need to be stored in
// their own file to maintain a small memory footprint.
pub type ConfigInstanceCache = FileCache<ConfigInstanceID, ConfigInstance>;
pub type ConfigInstanceDataCache = DirCache<ConfigInstanceID, serde_json::Value>;
