// standard library
use std::collections::HashMap;
use std::fmt::Debug;

// internal crates
use crate::cache::{
    concurrent::{
        ConcurrentCache,
        Worker,
        WorkerCommand,
        ConcurrentCacheKey,
        ConcurrentCacheValue,
    },
    entry::CacheEntry,
    errors::{
        CacheErr,
        CacheFileSysErr,
        CannotOverwriteCacheElement,
    },
    single_thread::{SingleThreadCache, CacheKey, CacheValue},
};
use crate::filesys::{file::File, path::PathExt};
use crate::trace;

// external crates
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct SingleThreadFileCache<K, V>
where
    K: CacheKey,
    V: CacheValue,
{
    file: File,
    _phantom: std::marker::PhantomData<K>,
    _phantom2: std::marker::PhantomData<V>,
}

impl<K, V> SingleThreadFileCache<K, V>
where
    K: CacheKey,
    V: CacheValue,
{
    pub async fn new(file: File) -> Result<Self, CacheErr> {
        if !file.exists() {
            let empty_cache: HashMap<K, CacheEntry<K, V>> = HashMap::new();
            file.write_json(
                &empty_cache,
                true,
                true,
            ).await.map_err(|e| {
                CacheErr::FileSysErr(CacheFileSysErr {
                    source: e,
                    trace: trace!(),
                })
            })?;
        }

        Ok(Self {
            file,
            _phantom: std::marker::PhantomData,
            _phantom2: std::marker::PhantomData,
        })
    }

    async fn read_cache(&self) -> Result<HashMap<K, CacheEntry<K, V>>, CacheErr> {
        self.file.read_json::<HashMap<K, CacheEntry<K, V>>>()
            .await
            .map_err(|e| {
                CacheErr::FileSysErr(CacheFileSysErr {
                    source: e,
                    trace: trace!(),
                })
            })
    }

    async fn write_cache(&self, cache: &HashMap<K, CacheEntry<K, V>>) -> Result<(), CacheErr> {
        self.file.write_json(
            cache,
            true,
            true,
        )
        .await
        .map_err(|e| {
                CacheErr::FileSysErr(CacheFileSysErr {
                    source: e,
                    trace: trace!(),
                })
            })
    }
}

impl<K, V> SingleThreadCache<K, V> for SingleThreadFileCache<K, V>
where
    K: CacheKey,
    V: CacheValue,
{
    async fn read_entry_impl(&self, key: &K) -> Result<Option<CacheEntry<K, V>>, CacheErr> {
        let cache = self.read_cache().await?;
        Ok(cache.get(key).cloned())
    }

    async fn write_entry_impl(&mut self, entry: &CacheEntry<K, V>, overwrite: bool) -> Result<(), CacheErr> {
        let mut cache = self.read_cache().await?;
        if !overwrite && cache.contains_key(&entry.key) {
            return Err(CacheErr::CannotOverwriteCacheElement(CannotOverwriteCacheElement {
                key: entry.key.to_string(),
                trace: trace!(),
            }));
        }
        cache.insert(entry.key.clone(), entry.clone());
        self.write_cache(&cache).await?;
        Ok(())
    }

    async fn delete_entry_impl(&mut self, key: &K) -> Result<(), CacheErr> {
        let mut cache = self.read_cache().await?;
        cache.remove(key);
        self.write_cache(&cache).await?;
        Ok(())
    }

    async fn size(&self) -> Result<usize, CacheErr> {
        let cache = self.read_cache().await?;
        Ok(cache.len())
    }

    async fn prune_invalid_entries(&self) -> Result<(), CacheErr> {
        Ok(())
    }

    async fn entries(&self) -> Result<Vec<CacheEntry<K, V>>, CacheErr> {
        let cache = self.read_cache().await?;
        Ok(cache.values().cloned().collect())
    }

    async fn values(&self) -> Result<Vec<V>, CacheErr> {
        let cache = self.read_cache().await?;
        Ok(cache.values().map(|v| v.value.clone()).collect())
    }

    async fn entry_map(&self) -> Result<HashMap<K, CacheEntry<K, V>>, CacheErr> {
        let cache = self.read_cache().await?;
        Ok(cache)
    }

    async fn value_map(&self) -> Result<HashMap<K, V>, CacheErr> {
        let cache = self.read_cache().await?;
        Ok(cache.into_iter().map(|(k, v)| (k, v.value)).collect())
    }
}

pub type FileCache<K, V> = ConcurrentCache<SingleThreadFileCache<K, V>, K, V>;

impl<K, V> FileCache<K, V>
where
    K: ConcurrentCacheKey,
    V: ConcurrentCacheValue,
{
    pub async fn spawn(
        file: File,
        buffer_size: usize,
    ) -> Result<(Self, JoinHandle<()>), CacheErr> {

        let (sender, receiver) = mpsc::channel::<WorkerCommand<K, V>>(buffer_size);
        let worker = Worker {
            cache: SingleThreadFileCache::new(file).await?,
            receiver,
        };
        let worker_handle = tokio::spawn(worker.run());
        Ok((Self::new(sender), worker_handle))
    }
}