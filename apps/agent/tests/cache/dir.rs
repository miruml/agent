// standard library
use std::path::PathBuf;

// internal crates
use config_agent::cache::dir::{SingleThreadDirCache, DirCache};
use config_agent::crud::prelude::*;
use config_agent::filesys::{dir::Dir, path::PathExt};
use crate::{concurrent_cache_tests, single_thread_cache_tests};

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use tokio::task::JoinHandle;


pub mod concurrent {
    use super::*;

    type TestCache = DirCache<String, String>;

    async fn spawn_cache() -> (TestCache, JoinHandle<()>) {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cache"));
        TestCache::spawn(32, dir.clone()).await.unwrap()
    }

    concurrent_cache_tests!(spawn_cache);

    #[tokio::test]
    async fn spawn() {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cache"));
        let _ = TestCache::spawn(32, dir.clone()).await.unwrap();
        // the directory should not exist yet
        assert!(dir.exists());

        // spawn again should not fail
        let _ = TestCache::spawn(32, dir.clone()).await.unwrap();
    }

    #[tokio::test]
    async fn prune_invalid_entries() {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cache"));
        let (cache, _) = TestCache::spawn(32, dir.clone()).await.unwrap();

        // write invalid json files to files in the cache directory
        let invalid_json_file = dir.file("invalid.json");
        invalid_json_file
            .write_string("invalid json", true, false)
            .await
            .unwrap();

        // create 10 entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.write(key, value, |_, _| true, false).await.unwrap();
        }

        // prune the cache
        cache.prune(10).await.unwrap();

        // invalid json file should be deleted
        assert!(!invalid_json_file.exists());

        // the cache should still have all ten entries
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = cache.read(key).await.unwrap();
            assert_eq!(value, format!("value{}", i));
        }
    }
}

pub mod single_thread {
    use super::*;

    type TestCache = SingleThreadDirCache<String, String>;

    async fn new_cache() -> TestCache {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cache"));
        TestCache::new(dir.clone()).await.unwrap()
    }

    #[tokio::test]
    async fn new() {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cache"));
        let _ = TestCache::new(dir.clone()).await.unwrap();
        assert!(dir.exists());

        // new should not fail
        let _ = TestCache::new(dir.clone()).await.unwrap();
    }

    single_thread_cache_tests!(new_cache);
}