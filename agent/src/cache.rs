// standard library
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

// internal crates
use crate::errors::Trace;
use crate::trace;

// ===================================== CACHE ==================================== //
pub trait CacheData where Self: Clone {}

#[derive(Clone)]
pub struct CacheEntry<DataT: CacheData> {
    data: DataT,
    expiration: SystemTime,
}

impl<DataT: CacheData> CacheEntry<DataT> {
    fn is_expired(&self) -> bool {
        SystemTime::now() > self.expiration
    }
}

#[derive(Default)]
pub struct Cache<DataT: CacheData> {
    cache: Arc<RwLock<HashMap<String, CacheEntry<DataT>>>>,
}

impl<DataT: CacheData> Cache<DataT> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn fetch(&self, key: &str) -> Result<DataT, CacheErr<'_, DataT>> {
        let cache_guard = self.cache.read().map_err(|e| {
            CacheErr::ReadLockErr {
                source: e,
                trace: trace!(),
            }
        })?;

        let entry = cache_guard.get(key).ok_or_else(|| {
            CacheErr::KeyNotFoundErr {
                key: key.to_string(),
                trace: trace!(),
            }
        })?;

        if entry.is_expired() {
            drop(cache_guard); // Release read lock before acquiring write lock
            let mut cache_guard = self.cache.write().map_err(|e| {
                CacheErr::WriteLockErr {
                    source: e,
                    trace: trace!(),
                }
            })?;
            cache_guard.remove(key);
            return Err(CacheErr::KeyNotFoundErr {
                key: key.to_string(),
                trace: trace!(),
            });
        }

        Ok(entry.data.clone())
    }

    pub fn store(
        &self,
        key: String,
        value: DataT,
        ttl: Duration,
    ) -> Result<(), CacheErr<'_, DataT>> {
        let entry = CacheEntry {
            data: value,
            expiration: SystemTime::now() + ttl,
        };

        let mut cache_guard = self.cache.write().map_err(|e| {
            CacheErr::WriteLockErr {
                source: e,
                trace: trace!(),
            }
        })?;

        cache_guard.insert(key, entry);
        Ok(())
    }

    pub fn remove(&self, key: &str) -> Result<(), CacheErr<'_, DataT>> {
        let mut cache_guard = self.cache.write().map_err(|e| {
            CacheErr::WriteLockErr {
                source: e,
                trace: trace!(),
            }
        })?;

        cache_guard.remove(key);
        Ok(())
    }
}

// ==================================== ERRORS ===================================== //
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CacheErr<'a, DataT: CacheData> {
    #[error("Failed to acquire read lock: {source}")]
    ReadLockErr {
        source: std::sync::PoisonError<std::sync::RwLockReadGuard<'a, HashMap<String, CacheEntry<DataT>>>>,
        trace: Box<Trace>,
    },
    #[error("Key not found: {key}")]
    KeyNotFoundErr {
        key: String,
        trace: Box<Trace>,
    },
    #[error("Failed to acquire write lock: {source}")]
    WriteLockErr {
        source: std::sync::PoisonError<std::sync::RwLockWriteGuard<'a, HashMap<String, CacheEntry<DataT>>>>,
        trace: Box<Trace>,
    },
    #[error("Failed to store key: {key}")]
    StoreErr {
        key: String,
        trace: Box<Trace>,
    },
    #[error("Failed to invalidate key: {key}")]
    InvalidateErr {
        key: String,
        trace: Box<Trace>,
    },
}