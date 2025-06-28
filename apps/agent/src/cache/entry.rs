// standard library
use std::fmt::Debug;
use std::cmp::Eq;
use std::hash::Hash;

// external crates
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry<K, V>
where
    K: ToString + Serialize,
    V: Clone + Serialize,
{
    pub key: K,
    pub value: V,
    pub is_dirty: bool,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

pub fn is_dirty_true<K, V>(_old: Option<&CacheEntry<K, V>>, _new: &V) -> bool
where
    K: Debug + Clone + Send + Sync + ToString + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    true
}

pub fn is_dirty_false<K, V>(_old: Option<&CacheEntry<K, V>>, _new: &V) -> bool
where
    K: Debug + Clone + Send + Sync + ToString + Serialize + DeserializeOwned + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    false
}