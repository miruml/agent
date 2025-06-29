// standard library
use std::path::PathBuf;

// internal crates
use config_agent::cache::{
    entry::is_dirty,
    dir::DirCache,
};
use config_agent::crud::prelude::*;
use config_agent::filesys::{dir::Dir, path::PathExt};
use crate::concurrent_cache_tests;

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use tokio::task::JoinHandle;

type TestCache = DirCache<String, String>;

async fn spawn_cache() -> (TestCache, JoinHandle<()>) {
    let dir = Dir::create_temp_dir("testing")
        .await
        .unwrap()
        .subdir(PathBuf::from("cache"));
    TestCache::spawn(dir.clone(), 32)
}

pub mod spawn {
    use super::*;

    #[tokio::test]
    async fn spawn() {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cache"));
        let _ = TestCache::spawn(dir.clone(), 32);
        // the directory should not exist yet
        assert!(!dir.exists());
    }
}


pub mod custom_prune {
    use super::*;

    #[tokio::test]
    async fn remove_invalid_entries() {
        let dir = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .subdir(PathBuf::from("cache"));
        let (cache, _) = TestCache::spawn(dir.clone(), 32);

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
            cache.write(key, value, is_dirty, false).await.unwrap();
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

concurrent_cache_tests!(spawn_cache);
