// standard library
use std::sync::Arc;
// intenal crates
use crate::filesys::file::File;
use crate::models::device_cfg::DeviceConfig;
use crate::storage::{cached_file::CachedFilePrivate, prelude::*};
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[derive(Clone, Debug)]
pub struct DeviceConfigFile {
    file: File,
    // cache the device configuration
    cache: Arc<DeviceConfig>,
    // store whether the device configuration file has updates which have not been sent
    // to the server
    synced: bool,
}

impl CachedFilePrivate<DeviceConfig> for DeviceConfigFile {
    fn set_cache(&mut self, cache: DeviceConfig) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<DeviceConfig> for DeviceConfigFile {
    fn init_struct(file: File, cache: DeviceConfig) -> Self {
        Self {
            file,
            cache: Arc::new(cache),
            synced: false, // assume the device configuration file is not synced
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    /// Return the name of the device configuration file
    fn file_name() -> &'static str {
        "config.json"
    }

    fn cache(&self) -> Arc<DeviceConfig> {
        self.cache.clone()
    }
}

impl Sync for DeviceConfigFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<DeviceConfig> for DeviceConfigFile {}
