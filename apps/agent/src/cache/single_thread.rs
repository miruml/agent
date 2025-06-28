// standard library
use std::cmp::Eq;
use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashMap;

// internal crates
use crate::cache::entry::CacheEntry;
use crate::cache::errors::{
    CacheElementNotFound, FoundTooManyCacheElements, CacheErr,
};
use crate::trace;

// external crates
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::info;


#[allow(async_fn_in_trait)]
pub trait SingleThreadCache<K, V>
where
    K: Debug + ToString + Serialize + DeserializeOwned + Eq + Hash,
    V: Debug + Clone + Serialize + DeserializeOwned 
{
// -------------------------------- CUSTOM METHODS --------------------------------- //
    async fn read_entry_impl(&self, key: &K) -> Result<Option<CacheEntry<K, V>>, CacheErr>;

    async fn write_entry_impl(
        &mut self,
        entry: &CacheEntry<K, V>,
        overwrite: bool,
    ) -> Result<(), CacheErr>;

    async fn delete_entry_impl(&mut self, key: &K) -> Result<(), CacheErr>;

    async fn size(&self) -> Result<usize, CacheErr>;

    async fn prune_invalid_entries(&self) -> Result<(), CacheErr>;

    async fn entries(&self) -> Result<Vec<CacheEntry<K, V>>, CacheErr>;

    async fn values(&self) -> Result<Vec<V>, CacheErr>;

    async fn entry_map(&self) -> Result<HashMap<K, CacheEntry<K, V>>, CacheErr>;

    async fn value_map(&self) -> Result<HashMap<K, V>, CacheErr>;

// -------------------------------- TRAIT METHODS ---------------------------------- //
    async fn read_entry_optional(
        &mut self,
        key: &K,
        update_last_accessed: bool,
    ) -> Result<Option<CacheEntry<K, V>>, CacheErr> {
        let mut entry = match self.read_entry_impl(key).await? {
            Some(entry) => entry,
            None => return Ok(None),
        };

        // update the last accessed time
        if update_last_accessed {
            self.update_last_accessed(&mut entry).await?;
        }

        Ok(Some(entry))
    }

    async fn read_entry(
        &mut self,
        key: &K,
        update_last_accessed: bool,
    ) -> Result<CacheEntry<K, V>, CacheErr> {
        let result = self.read_entry_optional(key, update_last_accessed).await?;
        match result {
            Some(entry) => Ok(entry),
            None => Err(CacheErr::CacheElementNotFound(CacheElementNotFound {
                msg: format!("Unable to find cache entry with key: '{}'", key.to_string()),
                trace: trace!(),
            })),
        }
    }

    async fn read_optional(&mut self, key: &K) -> Result<Option<V>, CacheErr> {
        let entry = self.read_entry_optional(key, true).await?;
        match entry {
            Some(entry) => Ok(Some(entry.value)),
            None => Ok(None),
        }
    }

    async fn read(&mut self, key: &K) -> Result<V, CacheErr> {
        Ok(self.read_entry(key, true).await?.value)
    }

    async fn write_entry(
        &mut self,
        entry: &CacheEntry<K, V>,
        overwrite: bool,
    ) -> Result<(), CacheErr> {
        self.write_entry_impl(entry, overwrite).await?;
        Ok(())
    }

    async fn write<F>(
        &mut self,
        key: K,
        value: V,
        is_dirty: F,
        overwrite: bool,
    ) -> Result<(), CacheErr>
    where
        F: Fn(Option<&CacheEntry<K, V>>, &V) -> bool + Send + Sync,
    {
        // if the entry already exists, keep the original created_at time
        let (created_at, last_accessed, is_dirty) = match self.read_entry_optional(&key, false).await? {
            Some(existing_entry) => (
                existing_entry.created_at,
                Utc::now(),
                is_dirty(Some(&existing_entry), &value),
            ),
            None => {
                let now = Utc::now();
                (now, now, is_dirty(None, &value))
            }
        };
        let entry = CacheEntry {
            key,
            value,
            created_at,
            last_accessed,
            is_dirty,
        };

        // write the entry
        self.write_entry(&entry, overwrite).await?;
        Ok(())
    }

    async fn update_last_accessed(&mut self, entry: &mut CacheEntry<K, V>) -> Result<(), CacheErr> {
        entry.last_accessed = Utc::now();
        self.write_entry(entry, true).await?;
        Ok(())
    }

    async fn delete(&mut self, key: &K) -> Result<(), CacheErr> {
        self.delete_entry_impl(key).await?;
        Ok(())
    }

    async fn prune(&mut self, max_size: usize) -> Result<(), CacheErr> {
        // check if there are too many files
        let size = self.size().await?;
        if size <= max_size {
            return Ok(());
        }

        info!(
            "Pruning cache {} from {:?} entries to {:?} entries...",
            std::any::type_name::<V>(),
            size,
            max_size
        );

        // prune the invalid entries first
        self.prune_invalid_entries().await?;

        // prune by last accessed time
        let mut entries = self.entries().await?;
        entries.sort_by_key(|entry| entry.last_accessed);
        let num_delete = entries.len() - max_size;
        for entry in entries.into_iter().take(num_delete) {
            self.delete(&entry.key).await?;
        }
        Ok(())
    }

    async fn find_entries_where<F>(
        &self,
        filter: F,
    ) -> Result<Vec<CacheEntry<K, V>>, CacheErr>
    where
        F: Fn(&CacheEntry<K, V>) -> bool,
    {
        let entries = self.entries().await?;
        let filtered_entries = entries
            .into_iter()
            .filter(|entry| filter(entry))
            .collect();
        Ok(filtered_entries)
    }

    async fn find_where<F>(
        &self,
        filter: F,
    ) -> Result<Vec<V>, CacheErr>
    where
        F: Fn(&V) -> bool,
    {
        let entries = self.entries().await?;
        let filtered_entries = entries
            .into_iter()
            .filter(|entry| filter(&entry.value))
            .map(|entry| entry.value)
            .collect();
        Ok(filtered_entries)
    }

    async fn find_one_entry_optional<F>(
        &self,
        filter_name: &str,
        filter: F,
    ) -> Result<Option<CacheEntry<K, V>>, CacheErr>
    where
        F: Fn(&CacheEntry<K, V>) -> bool,
    {
        let entries = self.find_entries_where(filter).await?;
        if entries.len() > 1 {
            return Err(CacheErr::FoundTooManyCacheElements(FoundTooManyCacheElements {
                expected_count: 1,
                actual_count: entries.len(),
                filter_name: filter_name.to_string(),
                trace: trace!(),
            }));
        }
        Ok(entries.into_iter().next())
    }

    async fn find_one_optional<F>(
        &self,
        filter_name: &str,
        filter: F,
    ) -> Result<Option<V>, CacheErr>
    where
        F: Fn(&V) -> bool,
    {
        let entries = self.find_where(filter).await?;
        if entries.len() > 1 {
            return Err(CacheErr::FoundTooManyCacheElements(FoundTooManyCacheElements {
                expected_count: 1,
                actual_count: entries.len(),
                filter_name: filter_name.to_string(),
                trace: trace!(),
            }));
        }
        Ok(entries.into_iter().next())
    }

    async fn find_one_entry<F>(
        &self,
        filter_name: &str,
        filter: F,
    ) -> Result<CacheEntry<K, V>, CacheErr>
    where
        F: Fn(&CacheEntry<K, V>) -> bool,
    {
        let entry = self.find_one_entry_optional(filter_name, filter).await?;
        match entry {
            Some(entry) => Ok(entry),
            None => Err(CacheErr::CacheElementNotFound(CacheElementNotFound {
                msg: format!("Unable to find cache entry with filter: '{}'", filter_name),
                trace: trace!(),
            })),
        }
    }

    async fn find_one<F>(
        &self,
        filter_name: &str,
        filter: F,
    ) -> Result<V, CacheErr>
    where
        F: Fn(&V) -> bool,
    {
        let opt_value = self.find_one_optional(filter_name, filter).await?;
        match opt_value {
            Some(value) => Ok(value),
            None => Err(CacheErr::CacheElementNotFound(CacheElementNotFound {
                msg: format!("Unable to find cache entry with filter: '{}'", filter_name),
                trace: trace!(),
            })),
        }
    }
}