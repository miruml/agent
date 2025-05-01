// standard crates
use std::path::PathBuf;

// internal crates
use config_agent::filesys::{dir::Dir, file::File, path::PathExt};
use config_agent::server::run::{run, RunServerOptions};
use config_agent::storage::{
    agent::Agent,
    digests::{ConfigSchemaDigests, ConfigSchemaDigestCache},
    concrete_configs::{ConcreteConfig, ConcreteConfigCache, ConcreteConfigCacheKey},
    layout::StorageLayout,
};

// external crates
use chrono::Utc;
use tokio::time::Duration;
use serde_json::json;

async fn prepare_valid_server_storage(dir: Dir) {
    let layout = StorageLayout::new(dir);

    // create a private key file
    let private_key_file = layout.auth_dir().private_key_file();
    private_key_file
        .write_string("test", false, false)
        .await
        .unwrap();

    // create the agent file
    let agent_file = layout.agent_file();
    let agent = Agent::default();
    agent_file.write_json(&agent, false, false).await.unwrap();
}

#[tokio::test]
async fn invalid_server_state_initialization() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let options = RunServerOptions {
        layout: StorageLayout::new(dir),
        ..Default::default()
    };
    tokio::time::timeout(Duration::from_secs(5), async move {
        run(options, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap_err();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn max_runtime_reached() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: StorageLayout::new(dir),
        max_runtime: Duration::from_millis(100),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // should safely run and shutdown in about 100ms
    tokio::time::timeout(Duration::from_secs(5), async move {
        run(options, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn idle_timeout_reached() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: StorageLayout::new(dir),
        idle_timeout: Duration::from_millis(100),
        idle_timeout_poll_interval: Duration::from_millis(10),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // should safely run and shutdown in about 100ms
    tokio::time::timeout(Duration::from_secs(5), async move {
        run(options, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn shutdown_signal_received() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: StorageLayout::new(dir),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // Create a channel for manual shutdown
    let (tx, rx) = tokio::sync::oneshot::channel();

    // Spawn the server in a task
    let server_handle = tokio::spawn(async move {
        run(options, async {
            let _ = rx.await;
        })
        .await
        .unwrap();
    });

    // Small delay to ensure server is running
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send shutdown signal
    tx.send(()).unwrap();

    // Wait for server to shutdown with timeout
    tokio::time::timeout(Duration::from_secs(5), server_handle)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn prune_config_schema_digest_cache() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let layout = StorageLayout::new(dir.clone());
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: layout.clone(),
        config_schema_digest_cache_max_size: 1,
        idle_timeout: Duration::from_millis(100),
        idle_timeout_poll_interval: Duration::from_millis(10),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // create some cache entries
    let cache_dir = layout.config_schema_digest_cache();
    let (cache, _) = ConfigSchemaDigestCache::spawn(cache_dir.clone());
    for i in 0..10 {
        cache.write(
            format!("test{}", i),
            ConfigSchemaDigests {
                raw: format!("test{}", i),
                resolved: format!("test{}", i),
            },
            false,
        ).await.unwrap();
    }

    // run the server
    tokio::time::timeout(Duration::from_secs(5), async move {
        run(options, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
    })
    .await
    .unwrap();

    // check that the cache is pruned
    let files = cache_dir.files().await.unwrap();
    assert_eq!(files.len(), 1);
}

#[tokio::test]
async fn prune_concrete_config_cache() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let layout = StorageLayout::new(dir.clone());
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: layout.clone(),
        concrete_config_cache_max_size: 1,
        idle_timeout: Duration::from_millis(100),
        idle_timeout_poll_interval: Duration::from_millis(10),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // create some cache entries
    let cache_dir = layout.concrete_config_cache();
    let (cache, _) = ConcreteConfigCache::spawn(cache_dir.clone());
    for i in 0..10 {
        cache.write(
            ConcreteConfigCacheKey {
                config_slug: format!("test{}", i),
                config_schema_digest: format!("test{}", i),
            },
            ConcreteConfig {
                id: format!("test{}", i),
                created_at: Utc::now().to_rfc3339(),
                client_id: "test".to_string(),
                config_schema_id: format!("test{}", i),
                concrete_config: json!({ "test": i }),
                config_slug: format!("test{}", i),
                config_schema_digest: format!("test{}", i),
            },
            false,
        ).await.unwrap();
    }

    // run the server
    tokio::time::timeout(Duration::from_secs(5), async move {
        run(options, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
    })
    .await
    .unwrap();

    // check that the cache is pruned
    let files = cache_dir.files().await.unwrap();
    for file in files.iter() {
        println!("{}", file.path().to_string_lossy());
    }
    assert_eq!(files.len(), 1);
}