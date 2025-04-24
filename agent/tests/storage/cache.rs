#[cfg(test)]
mod tests {
    // standard library
    use std::path::PathBuf;

    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt};
    use config_agent::storage::{
        digests::{ConfigSchemaDigestCache, ConfigSchemaDigests},
        errors::StorageErr,
        layout::StorageLayout,
    };

    // external crates
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

    pub mod read_optional {
        use super::*;

        #[tokio::test]
        async fn doesnt_exist() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            let key = "1234567890".to_string();
            let read_digests = cache.read_optional(key.clone()).await.unwrap();
            assert_eq!(read_digests, None);
        }

        #[tokio::test]
        async fn exists() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            let digests = ConfigSchemaDigests {
                raw: "1234567890".to_string(),
                resolved: "1234567890".to_string(),
            };
            let key = "1234567890".to_string();
            cache
                .write(key.clone(), digests.clone(), false)
                .await
                .unwrap();
            let read_digests = cache.read_optional(key.clone()).await.unwrap().unwrap();
            assert_eq!(read_digests, digests);
        }
    }

    pub mod read {
        use super::*;

        #[tokio::test]
        async fn doesnt_exist() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            assert!(matches!(
                cache.read("1234567890".to_string()).await.unwrap_err(),
                StorageErr::CacheElementNotFound { .. }
            ));
        }

        #[tokio::test]
        async fn exists() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
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
            let read_digests = cache.read(key.clone()).await.unwrap();
            assert_eq!(read_digests, digests);
        }
    }

    pub mod write {
        use super::*;

        #[tokio::test]
        async fn doesnt_exist_overwrite_false() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            let digests = ConfigSchemaDigests {
                raw: "1234567890".to_string(),
                resolved: "1234567890".to_string(),
            };
            let key = "1234567890".to_string();
            cache
                .write(key.clone(), digests.clone(), false)
                .await
                .unwrap();

            // the directory should exist now
            assert!(dir.exists());

            // reading the digests should return the digests
            let read_digests = cache.read(key.clone()).await.unwrap();
            assert_eq!(read_digests, digests);
        }

        #[tokio::test]
        async fn doesnt_exist_overwrite_true() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            let digests = ConfigSchemaDigests {
                raw: "1234567890".to_string(),
                resolved: "1234567890".to_string(),
            };
            let key = "1234567890".to_string();
            cache
                .write(key.clone(), digests.clone(), true)
                .await
                .unwrap();

            // the directory should exist now
            assert!(dir.exists());

            // reading the digests should return the digests
            let read_digests = cache.read(key.clone()).await.unwrap();
            assert_eq!(read_digests, digests);
        }

        #[tokio::test]
        async fn exists_overwrite_false() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
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
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            let digests = ConfigSchemaDigests {
                raw: "1234567890".to_string(),
                resolved: "1234567890".to_string(),
            };
            let key = "1234567890".to_string();
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
            let read_digests = cache.read(key.clone()).await.unwrap();
            assert_eq!(read_digests, digests);
        }

        #[tokio::test]
        #[ignore]
        async fn sandbox() {
            let layout = StorageLayout::default();
            let dir = layout.config_schema_digest_cache();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            let raw_digest = "47d47a5be146128845c5c7889707f65cc7356587662221289eb09aacdf05a7ea";
            let digests = ConfigSchemaDigests {
                raw: raw_digest.to_string(),
                resolved: "resolved-digest".to_string(),
            };

            cache
                .write(raw_digest.to_string(), digests.clone(), false)
                .await
                .unwrap();

            // reading the digests should return the digests
            let read_digests = cache.read(raw_digest.to_string()).await.unwrap();
            assert_eq!(read_digests, digests);
        }
    }

    pub mod delete {
        use super::*;

        #[tokio::test]
        async fn doesnt_exist() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            let key = "1234567890".to_string();
            cache.delete(key.clone()).await.unwrap();
        }

        #[tokio::test]
        async fn exists() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
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
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());
            cache.prune(10).await.unwrap();
        }

        #[tokio::test]
        async fn cache_equal_to_max_size() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());

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
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());

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
            let cache = ConfigSchemaDigestCache::spawn(dir.clone());

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
}
