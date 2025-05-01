// standard library
use std::path::PathBuf;

// internal crates
use config_agent::filesys::{dir::Dir, path::PathExt};
use config_agent::storage::{
    cache::CacheEntry,
    digests::{ConfigSchemaDigestCache, ConfigSchemaDigests},
    errors::StorageErr,
};

// external crates
use chrono::Utc;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub mod spawn {
    use super::*;

    #[tokio::test]
    async fn spawn() {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cfg_sch_digest_reg"));
        let _ = ConfigSchemaDigestCache::spawn(dir.clone());
        // the directory should not exist yet
        assert!(!dir.exists());
    }
}

pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn shutdown() {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cfg_sch_digest_reg"));
        let (cache, worker_handle) = ConfigSchemaDigestCache::spawn(dir.clone());
        cache.shutdown().await.unwrap();
        worker_handle.await.unwrap();
    }
}

pub mod read_entry_optional {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let key = "1234567890".to_string();
        let read_entry = cache.read_entry_optional(
            key.clone(),
            false
        ).await.unwrap();
        assert_eq!(read_entry, None);
    }

    #[tokio::test]
    async fn update_last_accessed_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();
        let before_read = Utc::now();
        let read_entry = cache.read_entry_optional(
            key.clone(),
            true
        ).await.unwrap().unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert!(read_entry.last_accessed > read_entry.created_at);
        assert!(read_entry.last_accessed > before_read);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value: digests,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
        };
        assert_eq!(read_entry, expected_entry);
    }

    #[tokio::test]
    async fn update_last_accessed_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();
        let read_entry = cache.read_entry_optional(
            key.clone(),
            false 
        ).await.unwrap().unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value: digests,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
        };
        assert_eq!(read_entry, expected_entry);
    }
}

pub mod read_entry {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        assert!(matches!(
            cache.read_entry(
                "1234567890".to_string(),
                false
            ).await.unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn update_last_accessed_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();
        let before_read = Utc::now();
        let read_entry = cache.read_entry(
            key.clone(),
            true
        ).await.unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert!(read_entry.last_accessed > read_entry.created_at);
        assert!(read_entry.last_accessed > before_read);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value: digests,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
        };
        assert_eq!(read_entry, expected_entry);
    }

    #[tokio::test]
    async fn update_last_accessed_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();
        let read_entry = cache.read_entry(
            key.clone(),
            false 
        ).await.unwrap();

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);

        // check the values
        let expected_entry = CacheEntry {
            key,
            value: digests,
            created_at: read_entry.created_at,
            last_accessed: read_entry.last_accessed,
        };
        assert_eq!(read_entry, expected_entry);
    }

}

pub mod read_optional {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let key = "1234567890".to_string();
        let read_digests = cache.read_optional(key.clone()).await.unwrap();
        assert_eq!(read_digests, None);
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();
        let before_read = Utc::now();
        let read_digests = cache.read_optional(key.clone()).await.unwrap().unwrap();
        assert_eq!(read_digests, digests);

        // check the last accessed time was properly set
        let after_read= Utc::now();
        let entry = cache.read_entry(
            key.clone(),
            false
        ).await.unwrap();
        assert!(entry.last_accessed > before_read);
        assert!(entry.last_accessed < after_read);
    }
}

pub mod read {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        assert!(matches!(
            cache.read("1234567890".to_string()).await.unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();

        // reading the digests should return the digests
        let before_read = Utc::now();
        let read_digests = cache.read(key.clone()).await.unwrap();
        assert_eq!(read_digests, digests);

        // check the last accessed time was properly set
        let after_read= Utc::now();
        let entry = cache.read_entry(
            key.clone(),
            false
        ).await.unwrap();
        assert!(entry.last_accessed > before_read);
        assert!(entry.last_accessed < after_read);
    }
}

pub mod write {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let before_write = Utc::now();
        let key = "1234567890".to_string();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert_eq!(read_entry.value, digests);

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);
    }

    #[tokio::test]
    async fn doesnt_exist_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        let before_write = Utc::now();
        cache
            .write(key.clone(), digests.clone(), true)
            .await
            .unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert_eq!(read_entry.value, digests);

        // check the timestamps
        assert!(read_entry.created_at > before_write);
        assert_eq!(read_entry.last_accessed, read_entry.created_at);
    }

    #[tokio::test]
    async fn exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();

        // should throw an error since already exists
        assert!(matches!(
            cache
                .write(key.clone(), digests.clone(), false)
                .await
                .unwrap_err(),
            StorageErr::FileSysErr { .. }
        ));
    }

    #[tokio::test]
    async fn exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        let before_creation = Utc::now();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();

        // should not throw an error since overwrite is true
        cache
            .write(key.clone(), digests.clone(), true)
            .await
            .unwrap();

        // reading the digests should return the digests
        let read_entry = cache.read_entry(key.clone(), false).await.unwrap();
        assert_eq!(read_entry.value, digests);

        // check the timestamps
        assert!(read_entry.created_at > before_creation);
        assert!(read_entry.last_accessed > read_entry.created_at);
    }
}

pub mod delete {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let key = "1234567890".to_string();
        cache.delete(key.clone()).await.unwrap();
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        let key = "1234567890".to_string();
        cache
            .write(key.clone(), digests.clone(), false)
            .await
            .unwrap();

        // should not throw an error since it exists
        cache.delete(key.clone()).await.unwrap();

        // the cache should not exist now
        assert!(matches!(
            cache.read(key.clone()).await.unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }
}

pub mod prune {
    use super::*;

    #[tokio::test]
    async fn empty_cache() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());
        cache.prune(10).await.unwrap();
    }

    #[tokio::test]
    async fn cache_equal_to_max_size() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let digests = ConfigSchemaDigests {
                raw: format!("raw{}", i),
                resolved: format!("resolved{}", i),
            };
            cache.write(key, digests, false).await.unwrap();
        }

        // prune the cache
        cache.prune(10).await.unwrap();

        // the cache should still have all ten entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let digests = cache.read(key).await.unwrap();
            let expected_digests = ConfigSchemaDigests {
                raw: format!("raw{}", i),
                resolved: format!("resolved{}", i),
            };
            assert_eq!(digests, expected_digests);
        }
    }

    #[tokio::test]
    async fn remove_invalid_entries() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());

        // write invalid json files to files in the cache directory
        let invalid_json_file = dir.file("invalid.json");
        invalid_json_file
            .write_string("invalid json", true, false)
            .await
            .unwrap();

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let digests = ConfigSchemaDigests {
                raw: format!("raw{}", i),
                resolved: format!("resolved{}", i),
            };
            cache.write(key, digests, false).await.unwrap();
        }

        // prune the cache
        cache.prune(10).await.unwrap();

        // invalid json file should be deleted
        assert!(!invalid_json_file.exists());

        // the cache should still have all ten entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let digests = cache.read(key).await.unwrap();
            let expected_digests = ConfigSchemaDigests {
                raw: format!("raw{}", i),
                resolved: format!("resolved{}", i),
            };
            assert_eq!(digests, expected_digests);
        }
    }

    #[tokio::test]
    async fn remove_oldest_entries() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let (cache, _) = ConfigSchemaDigestCache::spawn(dir.clone());

        // write invalid json files to files in the cache directory
        let invalid_json_file = dir.file("invalid.json");
        invalid_json_file
            .write_string("invalid json", true, false)
            .await
            .unwrap();

        // create 20 entries
        for i in 0..20 {
            let key = format!("key{}", i);
            let digests = ConfigSchemaDigests {
                raw: format!("raw{}", i),
                resolved: format!("resolved{}", i),
            };
            cache.write(key, digests, false).await.unwrap();
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
            let digests = cache.read(key).await.unwrap();
            let expected_digests = ConfigSchemaDigests {
                raw: format!("raw{}", i),
                resolved: format!("resolved{}", i),
            };
            assert_eq!(digests, expected_digests);
        }
    }
}
