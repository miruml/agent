// standard crates
use std::path::PathBuf;

// internal crates
use config_agent::filesys::{dir::Dir, file::File};
use config_agent::models::agent::Agent;
use config_agent::models::config_instance::{
    ActivityStatus, ConfigInstance, ErrorStatus, TargetStatus,
};
use config_agent::models::config_schema::ConfigSchema;
use config_agent::server::run::{run, RunServerOptions};
use config_agent::storage::{
    config_instances::{ConfigInstanceCache, ConfigInstanceDataCache},
    config_schemas::ConfigSchemaCache,
    digests::{ConfigSchemaDigestCache, ConfigSchemaDigests},
    layout::StorageLayout,
};

// external crates
use chrono::Utc;
use serde_json::json;
use tokio::time::Duration;

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
    let (cache, _) = ConfigSchemaDigestCache::spawn(16, cache_dir.clone())
        .await
        .unwrap();
    for i in 0..10 {
        cache
            .write(
                format!("test{}", i),
                ConfigSchemaDigests {
                    raw: format!("test{}", i),
                    resolved: format!("test{}", i),
                },
                |_, _| false,
                false,
            )
            .await
            .unwrap();
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
    assert_eq!(cache.size().await.unwrap(), 1);
}

#[tokio::test]
async fn prune_config_schema_cache() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let layout = StorageLayout::new(dir.clone());
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: layout.clone(),
        config_schema_cache_max_size: 1,
        idle_timeout: Duration::from_millis(100),
        idle_timeout_poll_interval: Duration::from_millis(10),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // create some cache entries
    let cache_file = layout.config_schema_cache();
    let (cache, _) = ConfigSchemaCache::spawn(16, cache_file.clone())
        .await
        .unwrap();
    for i in 0..10 {
        cache
            .write(
                format!("test{}", i),
                ConfigSchema {
                    id: format!("test{}", i),
                    version: 1,
                    digest: format!("test{}", i),
                    config_type_id: format!("test{}", i),
                    config_type_slug: Some(format!("test{}", i)),
                    created_at: Utc::now().to_rfc3339(),
                    created_by_id: None,
                },
                |_, _| false,
                false,
            )
            .await
            .unwrap();
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
    assert_eq!(cache.size().await.unwrap(), 1);
}

#[tokio::test]
async fn prune_config_instance_metadata_cache() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let layout = StorageLayout::new(dir.clone());
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: layout.clone(),
        config_instance_cache_max_size: 1,
        idle_timeout: Duration::from_millis(100),
        idle_timeout_poll_interval: Duration::from_millis(10),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // create some cache entries
    let cache_file = layout.config_instance_metadata_cache();
    let (cache, _) = ConfigInstanceCache::spawn(16, cache_file.clone())
        .await
        .unwrap();
    for i in 0..10 {
        cache
            .write(
                format!("test{}", i),
                ConfigInstance {
                    id: format!("test{}", i),
                    target_status: TargetStatus::Created,
                    activity_status: ActivityStatus::Created,
                    error_status: ErrorStatus::None,
                    relative_filepath: None,
                    patch_id: None,
                    created_by_id: None,
                    created_at: Utc::now(),
                    updated_by_id: None,
                    updated_at: Utc::now(),
                    device_id: "test".to_string(),
                    config_schema_id: format!("test{}", i),
                    attempts: 0,
                    cooldown_ends_at: Utc::now(),
                },
                |_, _| false,
                false,
            )
            .await
            .unwrap();
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
    assert_eq!(cache.size().await.unwrap(), 1);
}

#[tokio::test]
async fn prune_config_instance_data_cache() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let layout = StorageLayout::new(dir.clone());
    prepare_valid_server_storage(dir.clone()).await;
    let options = RunServerOptions {
        layout: layout.clone(),
        config_instance_cache_max_size: 1,
        idle_timeout: Duration::from_millis(100),
        idle_timeout_poll_interval: Duration::from_millis(10),
        socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        ..Default::default()
    };

    // create some cache entries
    let cache_dir = layout.config_instance_data_cache();
    let (cache, _) = ConfigInstanceDataCache::spawn(16, cache_dir.clone())
        .await
        .unwrap();
    for i in 0..10 {
        cache
            .write(
                format!("test{}", i),
                json!({ "test": i }),
                |_, _| false,
                false,
            )
            .await
            .unwrap();
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
    assert_eq!(cache.size().await.unwrap(), 1);
}
