// internal crates
use crate::storage::{cached_file::CachedFile, errors::StorageErr};
// external crates
use serde::{de::DeserializeOwned, Serialize};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

pub trait Sync {
    fn is_synced(&self) -> bool;
    fn set_synced(&mut self, synced: bool);
    fn mark_synced(&mut self) {
        self.set_synced(true);
    }
}

pub trait SyncedFile<T>
where
    T: Clone + Default + Serialize + DeserializeOwned,
    Self: CachedFile<T> + Sync,
{
    fn synced_write(&mut self, manifest: T) -> Result<(), StorageErr> {
        self.cached_write(manifest)?;
        self.set_synced(false);
        Ok(())
    }
}
