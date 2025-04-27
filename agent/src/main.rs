// internal
use config_agent::logs::{init, LogLevel};
use config_agent::server::run::{run, RunServerOptions};
use config_agent::storage::layout::StorageLayout;
use config_agent::storage::agent::assert_activated;

// external
use tracing::{error, info};
use tokio::signal::unix::signal;

#[tokio::main]
async fn main() {
    // initialize the logging
    let result = init(true, LogLevel::Debug);
    if let Err(e) = result {
        println!("Failed to initialize logging: {}", e);
    }

    // check the agent has been activated
    let layout = StorageLayout::default();
    let agent_file = layout.agent_file();
    if let Err(e) = assert_activated(&agent_file).await {
        error!("Agent is not yet activated: {}", e);
        return;
    }

    // run the server
    let result = run(
        RunServerOptions::default(),
        await_shutdown_signal(),
    ).await;
    if let Err(e) = result {
        error!("Failed to run the server: {}", e);
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
