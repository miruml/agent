// internal crates
use config_agent::cache::{
    file::{FileCache, SingleThreadFileCache},
};
use config_agent::filesys::{dir::Dir, path::PathExt};
use crate::{concurrent_cache_tests, single_thread_cache_tests};

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use tokio::task::JoinHandle;

pub mod concurrent {
    use super::*;

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

            // spawn again should not fail
            TestCache::spawn(file.clone(), 32).await.unwrap();
        }
    }

    concurrent_cache_tests!(spawn_cache);
}

pub mod single_thread {
    use super::*;

    type TestCache = SingleThreadFileCache<String, String>;

    async fn new_cache() -> TestCache {
        let file = Dir::create_temp_dir("testing")
            .await
            .unwrap()
            .file("cache.json");
        TestCache::new(file.clone()).await.unwrap()
    }

    pub mod new {
        use super::*;

        #[tokio::test]
        async fn new() {
            let file = Dir::create_temp_dir("testing")
                .await
                .unwrap()
                .file("cache.json");
            TestCache::new(file.clone()).await.unwrap();
            assert!(file.exists());

            // create again should not fail
            TestCache::new(file.clone()).await.unwrap();
        }
    }

    single_thread_cache_tests!(new_cache);
}
