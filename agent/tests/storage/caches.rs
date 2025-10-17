// internal crates
use miru_agent::filesys::dir::Dir;
use miru_agent::storage::caches::{CacheCapacities, Caches};
use miru_agent::storage::layout::StorageLayout;

pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn shutdown() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let layout = StorageLayout::new(dir);
        let capacities = CacheCapacities::default();
        let (caches, _) = Caches::init(&layout, capacities).await.unwrap();

        // shutdown the caches
        caches.shutdown().await.unwrap();
    }
}
