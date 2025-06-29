// internal crates
use config_agent::cache::{
    file::FileCache,
};
use config_agent::filesys::{dir::Dir, path::PathExt};
use crate::concurrent_cache_tests;

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use tokio::task::JoinHandle;

type TestCache = FileCache<String, String>;

async fn spawn_cache() -> (TestCache, JoinHandle<()>) {
    let file = Dir::create_temp_dir("testing")
        .await
        .unwrap()
        .file("cache.json");
    TestCache::spawn(file.clone(), 32).await.unwrap()
}

pub mod spawn {
    use super::*;

    #[tokio::test]
    async fn spawn() {
        let file = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .file("cache.json");
        TestCache::spawn(file.clone(), 32).await.unwrap();
        assert!(file.exists());
    }
}

concurrent_cache_tests!(spawn_cache);