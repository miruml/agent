// standard library
use std::sync::Arc;
// internal crates
use crate::filesys::{file::File, path::PathExt};
use crate::storage::errors::StorageErr;
use crate::trace;
// external crates
use serde::{de::DeserializeOwned, Serialize};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

pub(super) trait CachedFilePrivate<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    fn set_cache(&mut self, cfg: T);
}

#[allow(private_bounds)]
pub trait CachedFile<T>
where
    T: Clone + Default + Serialize + DeserializeOwned,
    Self: CachedFilePrivate<T> + Sized,
{
    // must be implemented by the struct
    fn init_struct(file: File, cache: T) -> Self;
    fn file(&self) -> &File;
    fn file_name() -> &'static str;
    fn file_permissions() -> u32;
    fn cache(&self) -> Arc<T>;

    /// Initialize the cached file at the given file. This file must exist with valid
    /// contents or an error will be thrown.
    fn new(file: File) -> Result<Self, StorageErr> {
        let cache = file.read_json::<T>().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;

        // initialize the struct with the read data
        let cached_file = Self::init_struct(file, cache);
        cached_file.validate()?;
        Ok(cached_file)
    }

    fn create(file: File, data: &T) -> Result<Self, StorageErr>
    where
        Self: Sized,
    {
        Self::validate_file_name(&file)?;
        file.write_json(data, true)
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Self::new(file)
    }

    fn validate(&self) -> Result<(), StorageErr> {
        Self::validate_file_name(self.file())?;
        self.file()
            .assert_exists()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(())
    }

    fn validate_name(name: &str) -> Result<(), StorageErr> {
        if name != Self::file_name() {
            return Err(StorageErr::InvalidFileName {
                name: name.to_string(),
                expected_name: Some(Self::file_name().to_string()),
                trace: trace!(),
            });
        }
        Ok(())
    }

    fn validate_file_name(file: &File) -> Result<(), StorageErr> {
        Self::validate_name(file.name().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?)
    }

    fn cached_write(&mut self, data: T) -> Result<(), StorageErr> {
        self.file()
            .write_json(&data, true)
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        self.set_cache(data);
        Ok(())
    }
}
