// standard crates
use std::path::PathBuf;

// internal crates
use config_agent::app::run::{run};
use config_agent::app::options::{AppOptions, LifecycleOptions, StorageOptions};
use config_agent::filesys::{dir::Dir, file::File};
use config_agent::storage::agent::Agent;
use config_agent::server::serve::ServerOptions;
use config_agent::storage::layout::StorageLayout;

// external crates
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
async fn invalid_app_state_initialization() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let options = AppOptions {
        storage: StorageOptions {
            layout: StorageLayout::new(dir),
            ..Default::default()
        },
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
    let options = AppOptions {
        storage: StorageOptions {
            layout: StorageLayout::new(dir),
            ..Default::default()
        },
        lifecycle: LifecycleOptions {
            max_runtime: Duration::from_millis(100),
            ..Default::default()
        },
        server: ServerOptions {
            socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        },
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
async fn not_socket_activated() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    let max_runtime = Duration::from_millis(100);
    prepare_valid_server_storage(dir.clone()).await;
    let options = AppOptions {
        storage: StorageOptions {
            layout: StorageLayout::new(dir),
            ..Default::default()
        },
        lifecycle: LifecycleOptions {
            is_socket_activated: false,
            max_runtime,
            ..Default::default()
        },
        server: ServerOptions {
            socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        },
        ..Default::default()
    };

    tokio::time::timeout(2 * max_runtime, async move {
        run(options, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
    })
    .await
    // unwrap err because the test should timeout
    .unwrap_err();
}

#[tokio::test]
async fn idle_timeout_reached() {
    let dir = Dir::create_temp_dir("testing").await.unwrap();
    prepare_valid_server_storage(dir.clone()).await;
    let options = AppOptions {
        storage: StorageOptions {
            layout: StorageLayout::new(dir),
            ..Default::default()
        },
        lifecycle: LifecycleOptions {
            idle_timeout: Duration::from_millis(100),
            idle_timeout_poll_interval: Duration::from_millis(10),
            ..Default::default()
        },
        server: ServerOptions {
            socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        },
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
    let options = AppOptions {
        storage: StorageOptions {
            layout: StorageLayout::new(dir),
            ..Default::default()
        },
        server: ServerOptions {
            socket_file: File::new(PathBuf::from("/tmp").join("miru.sock")),
        },
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