// internal
use config_agent::app::run::{run};
use config_agent::app::options::{AppOptions, LifecycleOptions};
use config_agent::logs::{init, LogOptions};
use config_agent::mqtt::client::ConnectAddress;
use config_agent::storage::agent::{assert_activated, Agent};
use config_agent::storage::layout::StorageLayout;
use config_agent::storage::settings::Settings;
use config_agent::utils::{has_version_flag, version_info};
use config_agent::workers::backend_sync::BackendSyncWorkerOptions;

// external
use tokio::signal::unix::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // print the version and exit if that is all that is requested
    if has_version_flag() {
        println!("{}", version_info());
        return;
    }

    // check the agent has been activated
    let layout = StorageLayout::default();
    let agent_file = layout.agent_file();
    if let Err(e) = assert_activated(&agent_file).await {
        error!("Agent is not yet activated: {}", e);
        return;
    }
    let agent = match agent_file.read_json::<Agent>().await {
        Ok(agent) => agent,
        Err(e) => {
            error!("Unable to read agent file: {}", e);
            return;
        }
    };

    // retrieve the settings files
    let settings_file = layout.settings_file();
    let settings = match settings_file.read_json::<Settings>().await {
        Ok(settings) => settings,
        Err(e) => {
            error!("Unable to read settings file: {}", e);
            return;
        }
    };


    // initialize the logging
    let log_options = LogOptions {
        log_level: settings.log_level,
        ..Default::default()
    };
    let result = init(log_options);
    if let Err(e) = result {
        println!("Failed to initialize logging: {e}");
    }

    // run the server
    let options = AppOptions {
        lifecycle: LifecycleOptions {
            is_socket_activated: settings.is_socket_activated,
            ..Default::default()
        },
        backend_base_url: settings.backend.base_url,
        enable_socket_server: settings.enable_socket_server,
        enable_backend_sync_worker: settings.enable_backend_sync_worker,
        backend_sync_worker: BackendSyncWorkerOptions {
            mqtt_broker_address: ConnectAddress {
                broker: settings.mqtt_broker.host,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    info!("Running the server with options: {:?}", options);
    let result = run(options, await_shutdown_signal()).await;
    if let Err(e) = result {
        error!("Failed to run the server: {e}");
    }
}

async fn await_shutdown_signal() {
    let mut sigterm = signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    let mut sigint = signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigterm.recv() => {
            info!("SIGTERM received, shutting down...");
        }
        _ = sigint.recv() => {
            info!("SIGINT received, shutting down...");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("received ctrl-c, shutting down...");
        }
    }
}
