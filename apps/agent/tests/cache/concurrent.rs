// internal crates
use config_agent::cache::{
    concurrent::ConcurrentCache, entry::CacheEntry, errors::CacheErr,
    single_thread::SingleThreadCache,
};
use config_agent::crud::errors::CrudErr;
use config_agent::crud::prelude::*;

// external crates
use chrono::Utc;
use std::future::Future;
use tokio::task::JoinHandle;

#[macro_export]
macro_rules! concurrent_cache_tests {
    ($spawn_cache:expr, $spawn_cache_with_capacity:expr) => {
        pub mod shutdown {
            use super::*;

            #[tokio::test]
            async fn test_shutdown() {
                $crate::cache::concurrent::shutdown::shutdown_impl($spawn_cache).await;
            }
        }

        pub mod size {
            use super::*;

            #[tokio::test]
            async fn test_size() {
                $crate::cache::concurrent::size::size_impl($spawn_cache).await;
            }
        }

        pub mod entry_map {
            use super::*;

            #[tokio::test]
            async fn test_entry_map() {
                $crate::cache::concurrent::entry_map::entry_map_impl($spawn_cache).await;
            }
        }

        pub mod value_map {
            use super::*;

            #[tokio::test]
            async fn test_value_map() {
                $crate::cache::concurrent::value_map::value_map_impl($spawn_cache).await;
            }
        }

        pub mod entries {
            use super::*;

            #[tokio::test]
            async fn test_entries() {
                $crate::cache::concurrent::entries::entries_impl($spawn_cache).await;
            }
        }

        pub mod values {
            use super::*;

            #[tokio::test]
            async fn test_values() {
                $crate::cache::concurrent::values::values_impl($spawn_cache).await;
            }
        }

        pub mod read_entry_optional {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist() {
                $crate::cache::concurrent::read_entry_optional::doesnt_exist_impl($spawn_cache)
                    .await;
            }

            #[tokio::test]
            async fn exists() {
                $crate::cache::concurrent::read_entry_optional::exists_impl($spawn_cache).await;
            }
        }

        pub mod read_entry {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist() {
                $crate::cache::concurrent::read_entry::doesnt_exist_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn exists() {
                $crate::cache::concurrent::read_entry::exists_impl($spawn_cache).await;
            }
        }

        pub mod read_optional {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist() {
                $crate::cache::concurrent::read_optional::doesnt_exist_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn exists() {
                $crate::cache::concurrent::read_optional::exists_impl($spawn_cache).await;
            }
        }

        pub mod read {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist() {
                $crate::cache::concurrent::read::doesnt_exist_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn exists() {
                $crate::cache::concurrent::read::exists_impl($spawn_cache).await;
            }
        }

        pub mod write {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist_overwrite_false() {
                $crate::cache::concurrent::write::doesnt_exist_overwrite_false_impl($spawn_cache)
                    .await;
            }

            #[tokio::test]
            async fn doesnt_exist_overwrite_true() {
                $crate::cache::concurrent::write::doesnt_exist_overwrite_true_impl($spawn_cache)
                    .await;
            }

            #[tokio::test]
            async fn exists_overwrite_false() {
                $crate::cache::concurrent::write::exists_overwrite_false_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn exists_overwrite_true() {
                $crate::cache::concurrent::write::exists_overwrite_true_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn trigger_prune() {
                $crate::cache::concurrent::write::trigger_prune_impl($spawn_cache_with_capacity)
                    .await;
            }
        }

        pub mod delete {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist() {
                $crate::cache::concurrent::delete::doesnt_exist_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn exists() {
                $crate::cache::concurrent::delete::exists_impl($spawn_cache).await;
            }
        }

        pub mod prune {
            use super::*;

            #[tokio::test]
            async fn empty_cache() {
                $crate::cache::concurrent::prune::empty_cache_impl($spawn_cache_with_capacity)
                    .await;
            }

            #[tokio::test]
            async fn cache_equal_to_max_size() {
                $crate::cache::concurrent::prune::cache_equal_to_max_size_impl(
                    $spawn_cache_with_capacity,
                )
                .await;
            }

            #[tokio::test]
            async fn remove_oldest_entries() {
                $crate::cache::concurrent::prune::remove_oldest_entries_impl(
                    $spawn_cache_with_capacity,
                )
                .await;
            }
        }

        pub mod find_entries_where {
            use super::*;

            #[tokio::test]
            async fn find_entries_where() {
                $crate::cache::concurrent::find_entries_where::find_entries_where_impl(
                    $spawn_cache,
                )
                .await;
            }
        }

        pub mod find_where {
            use super::*;

            #[tokio::test]
            async fn find_where() {
                $crate::cache::concurrent::find_where::find_where_impl($spawn_cache).await;
            }
        }

        pub mod find_one_entry_optional {
            use super::*;

            #[tokio::test]
            async fn find_one_entry_optional() {
                $crate::cache::concurrent::find_one_entry_optional::find_one_entry_optional_impl(
                    $spawn_cache,
                )
                .await;
            }
        }

        pub mod find_one_optional {
            use super::*;

            #[tokio::test]
            async fn find_one_optional() {
                $crate::cache::concurrent::find_one_optional::find_one_optional_impl($spawn_cache)
                    .await;
            }
        }

        pub mod find_one_entry {
            use super::*;

            #[tokio::test]
            async fn find_one_entry() {
                $crate::cache::concurrent::find_one_entry::find_one_entry_impl($spawn_cache).await;
            }
        }

        pub mod find_one {
            use super::*;

            #[tokio::test]
            async fn find_one() {
                $crate::cache::concurrent::find_one::find_one_impl($spawn_cache).await;
            }
        }

        pub mod get_dirty_entries {
            use super::*;

            #[tokio::test]
            async fn get_dirty_entries() {
                $crate::cache::concurrent::get_dirty_entries::get_dirty_entries_impl($spawn_cache)
                    .await;
            }
        }
    };
}

pub mod shutdown {
    use super::*;

    pub async fn shutdown_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, worker_handle) = spawn_cache().await;
        cache.shutdown().await.unwrap();
        worker_handle.await.unwrap();
    }
}

pub mod size {
    use super::*;

    pub async fn size_impl<F, Fut, SingleThreadCacheT>(new_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = new_cache().await;
        assert_eq!(cache.size().await.unwrap(), 0);

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }
        assert_eq!(cache.size().await.unwrap(), 10);

        // create 10 more entries
        for i in 0..10 {
            let j = i + 10;
            let key = format!("key{j}");
            let value = format!("value{j}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }
        assert_eq!(cache.size().await.unwrap(), 20);

        // overwrite 10 entries
        for i in 0..10 {
            let j = i + 5;
            let key = format!("key{j}");
            let value = format!("value{j}");
            cache.write(key, value, |_, _| true, true).await.unwrap();
        }
        assert_eq!(cache.size().await.unwrap(), 20);
    }
}

pub mod entry_map {
    use super::*;

    pub async fn entry_map_impl<F, Fut, SingleThreadCacheT>(new_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = new_cache().await;
        let result = cache.entry_map().await.unwrap();
        assert_eq!(result.len(), 0);

        // create 2 entries
        let key1 = "key1".to_string();
        let value1 = "value1".to_string();
        cache
            .write(key1.clone(), value1.clone(), |_, _| true, false)
            .await
            .unwrap();
        let key2 = "key2".to_string();
        let value2 = "value2".to_string();
        cache
            .write(key2.clone(), value2.clone(), |_, _| true, false)
            .await
            .unwrap();

        let result = cache.entry_map().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get(&key1).map(|e| e.value.clone()),
            Some(value1.clone())
        );
        assert_eq!(
            result.get(&key2).map(|e| e.value.clone()),
            Some(value2.clone())
        );
    }
}

pub mod value_map {
    use super::*;

    pub async fn value_map_impl<F, Fut, SingleThreadCacheT>(new_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = new_cache().await;
        let result = cache.value_map().await.unwrap();
        assert_eq!(result.len(), 0);

        // create 2 entries
        let key1 = "key1".to_string();
        let value1 = "value1".to_string();
        cache
            .write(key1.clone(), value1.clone(), |_, _| true, false)
            .await
            .unwrap();
        let key2 = "key2".to_string();
        let value2 = "value2".to_string();
        cache
            .write(key2.clone(), value2.clone(), |_, _| true, false)
            .await
            .unwrap();

        let result = cache.value_map().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&key1), Some(&value1));
        assert_eq!(result.get(&key2), Some(&value2));
    }
}

pub mod entries {
    use super::*;

    pub async fn entries_impl<F, Fut, SingleThreadCacheT>(new_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = new_cache().await;
        let result = cache.entries().await.unwrap();
        assert_eq!(result.len(), 0);

        // create 2 entries
        let key1 = "key1".to_string();
        let value1 = "value1".to_string();
        cache
            .write(key1.clone(), value1.clone(), |_, _| true, false)
            .await
            .unwrap();
        let key2 = "key2".to_string();
        let value2 = "value2".to_string();
        cache
            .write(key2.clone(), value2.clone(), |_, _| true, false)
            .await
            .unwrap();

        let mut result = cache.entries().await.unwrap();
        result.sort();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, key1);
        assert_eq!(result[0].value, value1);
        assert_eq!(result[1].key, key2);
        assert_eq!(result[1].value, value2);
    }
}

pub mod values {
    use super::*;

    pub async fn values_impl<F, Fut, SingleThreadCacheT>(new_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = new_cache().await;
        let result = cache.values().await.unwrap();
        assert_eq!(result.len(), 0);

        // create 2 entries
        let key1 = "key1".to_string();
        let value1 = "value1".to_string();
        cache
            .write(key1.clone(), value1.clone(), |_, _| true, false)
            .await
            .unwrap();
        let key2 = "key2".to_string();
        let value2 = "value2".to_string();
        cache
            .write(key2.clone(), value2.clone(), |_, _| true, false)
            .await
            .unwrap();

        let mut result = cache.values().await.unwrap();
        result.sort();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], value1);
        assert_eq!(result[1], value2);
    }
}

pub mod read_entry_optional {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let result = cache
            .read_entry_optional("1234567890".to_string())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        // spawn the cache
        let (cache, _) = spawn_cache().await;

        // write the entry
        let key = "key".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), |_, _| false, false)
            .await
            .unwrap();

        // read the entry
        let before_read = Utc::now();
        let read_entry = cache
            .read_entry_optional(key.clone())
            .await
            .unwrap()
            .unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert!(read_entry.last_accessed > read_entry.created_at);
        assert!(read_entry.last_accessed > before_read);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
            is_dirty: false,
        };
        assert_eq!(read_entry, expected_entry);
    }
}

pub mod read_entry {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        assert!(matches!(
            cache
                .read_entry("1234567890".to_string())
                .await
                .unwrap_err(),
            CacheErr::CacheElementNotFound { .. }
        ));
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        // spawn the cache
        let (cache, _) = spawn_cache().await;

        // write the entry
        let key = "key".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), |_, _| false, false)
            .await
            .unwrap();

        // read the entry
        let before_read = Utc::now();
        let read_entry = cache.read_entry(key.clone()).await.unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert!(read_entry.last_accessed > read_entry.created_at);
        assert!(read_entry.last_accessed > before_read);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
            is_dirty: false,
        };
        assert_eq!(read_entry, expected_entry);
    }
}

pub mod read_optional {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let read_value = cache.read_optional(key.clone()).await.unwrap();
        assert_eq!(read_value, None);
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), |_, _| true, false)
            .await
            .unwrap();
        let before_read = Utc::now();
        let read_value = cache.read(key.clone()).await.unwrap();
        assert_eq!(read_value, value);

        // check the last accessed time was properly set
        let after_read = Utc::now();
        let entries = cache.entries().await.unwrap();
        let entry = entries.iter().find(|e| e.key == key).unwrap();
        assert!(entry.last_accessed > before_read);
        assert!(entry.last_accessed < after_read);
    }
}

pub mod read {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        assert!(matches!(
            cache
                .read("1234567890".to_string())
                .await
                .unwrap_err(),
            CrudErr::CacheErr(ref e)
                if matches!(e.source, CacheErr::CacheElementNotFound { .. })
        ));
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), |_, _| true, false)
            .await
            .unwrap();

        // reading the digests should return the digests
        let before_read = Utc::now();
        let read_value = cache.read(key.clone()).await.unwrap();
        assert_eq!(read_value, value);

        // check the last accessed time was properly set
        let after_read = Utc::now();
        let entries = cache.entries().await.unwrap();
        let entry = entries.iter().find(|e| e.key == key).unwrap();
        assert!(entry.last_accessed > before_read);
        assert!(entry.last_accessed < after_read);
    }
}

pub mod write {
    use super::*;

    pub async fn doesnt_exist_overwrite_false_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let before_write = Utc::now();
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), |_, _| false, false)
            .await
            .unwrap();

        // check the value
        let entries = cache.entry_map().await.unwrap();
        let read_entry = entries.get(&key).unwrap();
        assert_eq!(read_entry.value, value);

        // check the is_dirty flag
        assert!(!read_entry.is_dirty);

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);
    }

    pub async fn doesnt_exist_overwrite_true_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), |_, _| true, true)
            .await
            .unwrap();

        // check the value
        let entries = cache.entry_map().await.unwrap();
        let read_entry = entries.get(&key).unwrap();
        assert_eq!(read_entry.value, value);

        // check the is_dirty flag
        assert!(read_entry.is_dirty);

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);
    }

    pub async fn exists_overwrite_false_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), |_, _| true, false)
            .await
            .unwrap();

        // should throw an error since already exists
        assert!(matches!(
            cache
                .write(key.clone(), value.clone(), |_, _| true, false)
                .await
                .unwrap_err(),
            CacheErr::CannotOverwriteCacheElement { .. }
        ));
    }

    pub async fn exists_overwrite_true_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        let before_creation = Utc::now();
        cache
            .write(key.clone(), value.clone(), |_, _| true, false)
            .await
            .unwrap();

        // should not throw an error since overwrite is true
        cache
            .write(key.clone(), value.clone(), |_, _| false, true)
            .await
            .unwrap();

        // check the value
        let entries = cache.entry_map().await.unwrap();
        let read_entry = entries.get(&key).unwrap();
        assert_eq!(read_entry.value, value);

        // check the is_dirty flag
        assert!(!read_entry.is_dirty);

        // check the timestamps
        assert!(read_entry.created_at > before_creation);
        assert!(read_entry.last_accessed > read_entry.created_at);
    }

    pub async fn trigger_prune_impl<F, Fut, SingleThreadCacheT>(new_cache: F)
    where
        F: Fn(usize) -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = new_cache(10).await;

        // create 11 entries
        for i in 0..11 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        // each write should trigger a prune
        for i in 11..20 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
            assert_eq!(cache.size().await.unwrap(), 11);
        }
    }
}

pub mod delete {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        cache.delete(key.clone()).await.unwrap();
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), |_, _| false, false)
            .await
            .unwrap();

        // should not throw an error since it exists
        cache.delete(key.clone()).await.unwrap();
        let error = cache.read(key.clone()).await.unwrap_err();
        assert!(matches!(
            error,
            CrudErr::CacheErr(ref e)
                if matches!(e.source, CacheErr::CacheElementNotFound { .. })
        ));
    }
}

pub mod prune {
    use super::*;

    pub async fn empty_cache_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn(usize) -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache(10).await;
        cache.prune().await.unwrap();
    }

    pub async fn cache_equal_to_max_size_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn(usize) -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache(10).await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        // prune the cache
        cache.prune().await.unwrap();

        // the cache should still have all ten entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = cache.read(key).await.unwrap();
            assert_eq!(value, format!("value{i}"));
        }
    }

    pub async fn remove_oldest_entries_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn(usize) -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache(10).await;

        // create 20 entries
        for i in 0..20 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        // prune the cache
        cache.prune().await.unwrap();

        // first 10 entries should be deleted since they are the oldest
        for i in 0..10 {
            let key = format!("key{i}");
            cache.read(key).await.unwrap_err();
        }

        // last 10 entries should still exist
        for i in 10..20 {
            let key = format!("key{i}");
            let value = cache.read(key).await.unwrap();
            assert_eq!(value, format!("value{i}"));
        }
    }
}

pub mod find_entries_where {
    use super::*;

    pub async fn find_entries_where_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        let after_write = Utc::now();

        // no entries found
        let found = cache
            .find_entries_where(|entry| entry.key == "key10")
            .await
            .unwrap();
        assert!(found.is_empty());

        // one entry found
        let found = cache
            .find_entries_where(|entry| entry.key == "key5")
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].key, "key5");

        // check the last accessed time was properly set
        assert!(found[0].last_accessed > after_write);

        // multiple entries found
        let found = cache
            .find_entries_where(|entry| entry.key != "key5")
            .await
            .unwrap();
        assert_eq!(found.len(), 9);
    }
}

pub mod find_where {
    use super::*;

    pub async fn find_where_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        let after_write = Utc::now();

        // no entries found
        let found = cache.find_where(|value| value == "value10").await.unwrap();
        assert!(found.is_empty());

        // one entry found
        let found = cache.find_where(|value| value == "value5").await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "value5");

        // check the last accessed time was properly set
        let entries = cache.entries().await.unwrap();
        let entry = entries.iter().find(|e| e.value == "value5").unwrap();
        assert!(entry.last_accessed > after_write);

        // multiple entries found
        let found = cache.find_where(|value| value != "value5").await.unwrap();
        assert_eq!(found.len(), 9);
    }
}

pub mod find_one_entry_optional {
    use super::*;

    pub async fn find_one_entry_optional_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        let after_write = Utc::now();

        // no entries found
        let found = cache
            .find_one_entry_optional("key10", |entry| entry.key == "key10")
            .await
            .unwrap();
        assert!(found.is_none());

        // one entry found
        let found = cache
            .find_one_entry_optional("key5", |entry| entry.key == "key5")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.clone().unwrap().key, "key5");

        // check the last accessed time was properly set
        assert!(found.unwrap().last_accessed > after_write);

        // multiple entries found
        let err = cache
            .find_one_entry_optional("not key5", |entry| entry.key != "key5")
            .await
            .unwrap_err();
        assert!(matches!(err, CacheErr::FoundTooManyCacheElements { .. }));
    }
}

pub mod find_one_optional {
    use super::*;

    pub async fn find_one_optional_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        let after_write = Utc::now();

        // no entries found
        let found = cache
            .find_one_optional("value10", |value| value == "value10")
            .await
            .unwrap();
        assert!(found.is_none());

        // one entry found
        let found = cache
            .find_one_optional("value5", |value| value == "value5")
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap(), "value5");

        // check the last accessed time was properly set
        let entries = cache.entries().await.unwrap();
        let entry = entries.iter().find(|e| e.value == "value5").unwrap();
        assert!(entry.last_accessed > after_write);

        // multiple entries found
        let err = cache
            .find_one_optional("not value5", |value| value != "value5")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            CrudErr::CacheErr(ref e)
                if matches!(e.source, CacheErr::FoundTooManyCacheElements{ .. })
        ));
    }
}

pub mod find_one_entry {
    use super::*;

    pub async fn find_one_entry_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        let after_write = Utc::now();

        // no entries found
        let error = cache
            .find_one_entry("key10", |entry| entry.key == "key10")
            .await
            .unwrap_err();
        assert!(matches!(error, CacheErr::CacheElementNotFound { .. }));

        // one entry found
        let found = cache
            .find_one_entry("key5", |entry| entry.key == "key5")
            .await
            .unwrap();
        assert_eq!(found.key, "key5");

        // check the last accessed time was properly set
        assert!(found.last_accessed > after_write);

        // multiple entries found
        let err = cache
            .find_one_entry("not key5", |entry| entry.key != "key5")
            .await
            .unwrap_err();
        assert!(matches!(err, CacheErr::FoundTooManyCacheElements { .. }));
    }
}

pub mod find_one {
    use super::*;

    pub async fn find_one_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        let after_write = Utc::now();

        // no entries found
        let error = cache
            .find_one("value10", |value| value == "value10")
            .await
            .unwrap_err();
        assert!(
            matches!(error, CrudErr::CacheErr(ref e) if matches!(e.source, CacheErr::CacheElementNotFound { .. }))
        );

        // one entry found
        let found = cache
            .find_one("value5", |value| value == "value5")
            .await
            .unwrap();
        assert_eq!(found, "value5");

        // check the last accessed time was properly set
        let entries = cache.entries().await.unwrap();
        let entry = entries.iter().find(|e| e.value == "value5").unwrap();
        assert!(entry.last_accessed > after_write);

        // multiple entries found
        let err = cache
            .find_one("not value5", |value| value != "value5")
            .await
            .unwrap_err();
        assert!(
            matches!(err, CrudErr::CacheErr(ref e) if matches!(e.source, CacheErr::FoundTooManyCacheElements{ .. }))
        );
    }
}

pub mod get_dirty_entries {
    use super::*;

    pub async fn get_dirty_entries_impl<F, Fut, SingleThreadCacheT>(spawn_cache: F)
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<
            Output = (
                ConcurrentCache<SingleThreadCacheT, String, String>,
                JoinHandle<()>,
            ),
        >,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        // get dirty entries
        let dirty_entries = cache.get_dirty_entries().await.unwrap();
        assert_eq!(dirty_entries.len(), 10);

        // add 10 more entries which are not dirty
        for i in 10..20 {
            let key = format!("key{i}");
            let value = format!("value{i}");
            cache.write(key, value, |_, _| false, false).await.unwrap();
        }

        // dirty entries should be the same as before
        let dirty_entries = cache.get_dirty_entries().await.unwrap();
        assert_eq!(dirty_entries.len(), 10);
    }
}
