// standard library
use std::sync::Arc;
// internal crates
use crate::filesys::{errors::FileSysErr, file::File};
// external crates
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct CachedFile<T>
where
    T: Serialize + DeserializeOwned,
{
    pub file: File,
    cache: Arc<T>,
}

impl<T> CachedFile<T>
where
    T: Serialize + DeserializeOwned,
{
    pub async fn new(file: File) -> Result<Self, FileSysErr> {
        let cache = file.read_json::<T>().await?;

        // initialize the struct with the read data
        let cached_file = Self {
            file,
            cache: Arc::new(cache),
        };
        Ok(cached_file)
    }

    pub async fn new_with_default(file: File, default: T) -> Result<Self, FileSysErr> {
        let result = Self::new(file.clone()).await;
        match result {
            Ok(cached_file) => Ok(cached_file),
            Err(_) => Self::create(file, &default, true).await,
        }
    }

    pub async fn create(file: File, data: &T, overwrite: bool) -> Result<Self, FileSysErr>
    where
        Self: Sized,
    {
        file.write_json(data, overwrite, true).await?;
        Self::new(file).await
    }

    pub fn read(&self) -> Arc<T> {
        self.cache.clone()
    }

    pub async fn write(&mut self, data: T) -> Result<(), FileSysErr> {
        self.file.write_json(&data, true, true).await?;
        self.cache = Arc::new(data);
        Ok(())
    }
}
