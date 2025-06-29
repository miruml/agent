// internal crates
use config_agent::cache::{
    concurrent::ConcurrentCache,
    entry::{CacheEntry, is_not_dirty, is_dirty},
    errors::CacheErr,
    single_thread::SingleThreadCache,
};
use config_agent::crud::prelude::*;
use config_agent::crud::errors::{CrudErr, CrudCacheErr};

// external crates
use chrono::Utc;
use tokio::task::JoinHandle;
use std::future::Future;

#[macro_export]
macro_rules! concurrent_cache_tests {
    ($spawn_cache:expr) => {

        pub mod shutdown {
            use super::*;

            #[tokio::test]
            async fn test_shutdown() {
                $crate::cache::concurrent::shutdown::test_shutdown_impl($spawn_cache).await;
            }
        }

        pub mod read_entry_optional {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist() {
                $crate::cache::concurrent::read_entry_optional::doesnt_exist_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn update_last_accessed_false() {
                $crate::cache::concurrent::read_entry_optional::update_last_accessed_false_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn update_last_accessed_true() {
                $crate::cache::concurrent::read_entry_optional::update_last_accessed_true_impl($spawn_cache).await;
            }
        }

        pub mod read_entry {
            use super::*;

            #[tokio::test]
            async fn doesnt_exist() {
                $crate::cache::concurrent::read_entry::doesnt_exist_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn update_last_accessed_false() {
                $crate::cache::concurrent::read_entry::update_last_accessed_false_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn update_last_accessed_true() {
                $crate::cache::concurrent::read_entry::update_last_accessed_true_impl($spawn_cache).await;
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
                $crate::cache::concurrent::write::doesnt_exist_overwrite_false_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn doesnt_exist_overwrite_true() {
                $crate::cache::concurrent::write::doesnt_exist_overwrite_true_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn exists_overwrite_false() {
                $crate::cache::concurrent::write::exists_overwrite_false_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn exists_overwrite_true() {
                $crate::cache::concurrent::write::exists_overwrite_true_impl($spawn_cache).await;
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
                $crate::cache::concurrent::prune::empty_cache_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn cache_equal_to_max_size() {
                $crate::cache::concurrent::prune::cache_equal_to_max_size_impl($spawn_cache).await;
            }

            #[tokio::test]
            async fn remove_oldest_entries() {
                $crate::cache::concurrent::prune::remove_oldest_entries_impl($spawn_cache).await;
            }
        }

        pub mod find_entries_where {
            use super::*;

            #[tokio::test]
            async fn find_entries_where() {
                $crate::cache::concurrent::find_entries_where::find_entries_where_impl($spawn_cache).await;
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
                $crate::cache::concurrent::find_one_entry_optional::find_one_entry_optional_impl($spawn_cache).await;
            }
        }

        pub mod find_one_optional {
            use super::*;

            #[tokio::test]
            async fn find_one_optional() {
                $crate::cache::concurrent::find_one_optional::find_one_optional_impl($spawn_cache).await;
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
    }
}

pub mod shutdown {
    use super::*;

    pub async fn test_shutdown_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, worker_handle) = spawn_cache().await;
        cache.shutdown().await.unwrap();
        worker_handle.await.unwrap();
    }
}

pub mod read_entry_optional {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let result = cache
            .read_entry_optional("1234567890".to_string(), false)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    pub async fn update_last_accessed_true_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        // spawn the cache
        let (cache, _) = spawn_cache().await;

        // write the entry
        let key = "key".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), is_not_dirty, false)
            .await
            .unwrap();

        // read the entry
        let before_read = Utc::now();
        let read_entry = cache
            .read_entry_optional(key.clone(), true)
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

    pub async fn update_last_accessed_false_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "key".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), is_dirty, false)
            .await
            .unwrap();
        let read_entry = cache
            .read_entry_optional(key.clone(), false)
            .await
            .unwrap()
            .unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
            is_dirty: true,
        };
        assert_eq!(read_entry, expected_entry);
    }
}

pub mod read_entry {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        assert!(matches!(
            cache
                .read_entry("1234567890".to_string(), false)
                .await
                .unwrap_err(),
            CacheErr::CacheElementNotFound { .. }
        ));
    }

    pub async fn update_last_accessed_true_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        // spawn the cache
        let (cache, _) = spawn_cache().await;

        // write the entry
        let key = "key".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), is_not_dirty, false)
            .await
            .unwrap();

        // read the entry
        let before_read = Utc::now();
        let read_entry = cache
            .read_entry(key.clone(), true)
            .await
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

    pub async fn update_last_accessed_false_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "key".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), is_dirty, false)
            .await
            .unwrap();
        let read_entry = cache
            .read_entry(key.clone(), false)
            .await
            .unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
            is_dirty: true,
        };
        assert_eq!(read_entry, expected_entry);
    }
}

pub mod read_optional {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let read_value = cache.read_optional(key.clone()).await.unwrap();
        assert_eq!(read_value, None);
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), is_dirty, false)
            .await
            .unwrap();
        let before_read = Utc::now();
        let read_value = cache.read_optional(key.clone()).await.unwrap().unwrap();
        assert_eq!(read_value, value);

        // check the last accessed time was properly set
        let after_read = Utc::now();
        let entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert!(entry.last_accessed > before_read);
        assert!(entry.last_accessed < after_read);
    }
}

pub mod read {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        assert!(matches!(
            cache
                .read("1234567890".to_string())
                .await
                .unwrap_err(),
            CrudErr::CacheErr(CrudCacheErr{
                source: CacheErr::CacheElementNotFound { .. },
                ..
            })
        ));
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), is_dirty, false)
            .await
            .unwrap();

        // reading the digests should return the digests
        let before_read = Utc::now();
        let read_value = cache.read(key.clone()).await.unwrap();
        assert_eq!(read_value, value);

        // check the last accessed time was properly set
        let after_read = Utc::now();
        let entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert!(entry.last_accessed > before_read);
        assert!(entry.last_accessed < after_read);
    }
}

pub mod write {
    use super::*;

    pub async fn doesnt_exist_overwrite_false_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let before_write = Utc::now();
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), is_not_dirty, false)
            .await
            .unwrap();

        // check the value
        let read_entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert_eq!(read_entry.value, value);

        // check the is_dirty flag
        assert!(!read_entry.is_dirty);

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);
    }

    pub async fn doesnt_exist_overwrite_true_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), value.clone(), is_dirty, true)
            .await
            .unwrap();

        // check the value
        let read_entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert_eq!(read_entry.value, value);

        // check the is_dirty flag
        assert!(read_entry.is_dirty);

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);
    }

    pub async fn exists_overwrite_false_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), is_dirty, false)
            .await
            .unwrap();

        // should throw an error since already exists
        assert!(matches!(
            cache
                .write(key.clone(), value.clone(), is_dirty, false)
                .await
                .unwrap_err(),
            CacheErr::CannotOverwriteCacheElement { .. }
        ));
    }

    pub async fn exists_overwrite_true_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        let before_creation = Utc::now();
        cache
            .write(key.clone(), value.clone(), is_dirty, false)
            .await
            .unwrap();

        // should not throw an error since overwrite is true
        cache
            .write(key.clone(), value.clone(), is_not_dirty, true)
            .await
            .unwrap();

        // check the value
        let read_entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert_eq!(read_entry.value, value);

        // check the is_dirty flag
        assert!(!read_entry.is_dirty);

        // check the timestamps
        assert!(read_entry.created_at > before_creation);
        assert!(read_entry.last_accessed > read_entry.created_at);
    }
}

pub mod delete {
    use super::*;

    pub async fn doesnt_exist_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        cache.delete(key.clone()).await.unwrap();
    }

    pub async fn exists_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        let key = "1234567890".to_string();
        let value = "value".to_string();
        cache
            .write(key.clone(), value.clone(), is_not_dirty, false)
            .await
            .unwrap();

        // should not throw an error since it exists
        cache.delete(key.clone()).await.unwrap();

        // the cache should not exist now
        assert!(matches!(
            cache.read(key.clone()).await.unwrap_err(),
            CrudErr::CacheErr(CrudCacheErr{
                source: CacheErr::CacheElementNotFound { .. },
                ..
            })
        ));
    }
}

pub mod prune {
    use super::*;

    pub async fn empty_cache_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;
        cache.prune(10).await.unwrap();
    }

    pub async fn cache_equal_to_max_size_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // prune the cache
        cache.prune(10).await.unwrap();

        // the cache should still have all ten entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = cache.read(key).await.unwrap();
            assert_eq!(value, format!("value{}", i));
        }
    }

    pub async fn remove_oldest_entries_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 20 entries
        for i in 0..20 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // prune the cache
        cache.prune(10).await.unwrap();

        // first 10 entries should be deleted since they are the oldest
        for i in 0..10 {
            let key = format!("key{}", i);
            cache.read(key).await.unwrap_err();
        }

        // last 10 entries should still exist
        for i in 10..20 {
            let key = format!("key{}", i);
            let value = cache.read(key).await.unwrap();
            assert_eq!(value, format!("value{}", i));
        }
    }
}

pub mod find_entries_where {
    use super::*;

    pub async fn find_entries_where_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // no entries found
        let found = cache.find_entries_where(|entry| entry.key == "key10").await.unwrap();
        assert!(found.is_empty());

        // one entry found
        let found = cache.find_entries_where(|entry| entry.key == "key5").await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].key, "key5");

        // multiple entries found
        let found = cache.find_entries_where(|entry| entry.key != "key5").await.unwrap();
        assert_eq!(found.len(), 9);
    }
}

pub mod find_where {
    use super::*;

    pub async fn find_where_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // no entries found
        let found = cache.find_where(|value| value == "value10").await.unwrap();
        assert!(found.is_empty());

        // one entry found
        let found = cache.find_where(|value| value == "value5").await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "value5");

        // multiple entries found
        let found = cache.find_where(|value| value != "value5").await.unwrap();
        assert_eq!(found.len(), 9);
    }
}

pub mod find_one_entry_optional {
    use super::*;

    pub async fn find_one_entry_optional_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // no entries found
        let found = cache.find_one_entry_optional("key10", |entry| entry.key == "key10").await.unwrap();
        assert!(found.is_none());

        // one entry found
        let found = cache.find_one_entry_optional("key5", |entry| entry.key == "key5").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().key, "key5");

        // multiple entries found
        let err = cache.find_one_entry_optional("not key5", |entry| entry.key != "key5").await.unwrap_err();
        assert!(matches!(err, CacheErr::FoundTooManyCacheElements { .. }));
    }
}

pub mod find_one_optional {
    use super::*;

    pub async fn find_one_optional_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // no entries found
        let found = cache.find_one_optional("value10", |value| value == "value10").await.unwrap();
        assert!(found.is_none());

        // one entry found
        let found = cache.find_one_optional("value5", |value| value == "value5").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap(), "value5");

        // multiple entries found
        let err = cache.find_one_optional("not value5", |value| value != "value5").await.unwrap_err();
        assert!(matches!(err, CrudErr::CacheErr(CrudCacheErr{
            source: CacheErr::FoundTooManyCacheElements { .. },
            ..
        })));
    }
}

pub mod find_one_entry {
    use super::*;

    pub async fn find_one_entry_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // no entries found
        let error = cache.find_one_entry("key10", |entry| entry.key == "key10").await.unwrap_err();
        assert!(matches!(error, CacheErr::CacheElementNotFound { .. }));

        // one entry found
        let found = cache.find_one_entry("key5", |entry| entry.key == "key5").await.unwrap();
        assert_eq!(found.key, "key5");

        // multiple entries found
        let err = cache.find_one_entry("not key5", |entry| entry.key != "key5").await.unwrap_err();
        assert!(matches!(err, CacheErr::FoundTooManyCacheElements { .. }));
    }
}

pub mod find_one {
    use super::*;

    pub async fn find_one_impl<F, Fut, SingleThreadCacheT>(
        spawn_cache: F,
    )
    where
        F: Fn() -> Fut + Clone,
        Fut: Future<Output = (ConcurrentCache<SingleThreadCacheT, String, String>, JoinHandle<()>)>,
        SingleThreadCacheT: SingleThreadCache<String, String>,
    {
        let (cache, _) = spawn_cache().await;

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, is_dirty, false).await.unwrap();
        }

        // no entries found
        let error = cache.find_one("value10", |value| value == "value10").await.unwrap_err();
        assert!(matches!(error, CrudErr::CacheErr(CrudCacheErr{
            source: CacheErr::CacheElementNotFound { .. },
            ..
        })));

        // one entry found
        let found = cache.find_one("value5", |value| value == "value5").await.unwrap();
        assert_eq!(found, "value5");

        // multiple entries found
        let err = cache.find_one("not value5", |value| value != "value5").await.unwrap_err();
        assert!(matches!(err, CrudErr::CacheErr(CrudCacheErr{
            source: CacheErr::FoundTooManyCacheElements { .. },
            ..
        })));
    }
}