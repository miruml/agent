// internal
use config_agent::server::api::{server, init_state};
use config_agent::logs::{init, LogLevel};
use config_agent::storage::layout::StorageLayout;

// external
use tracing::error;

#[tokio::main]
async fn main() {
    // initialize the logging
    let result = init(true, LogLevel::Debug);
    if let Err(e) = result {
        println!("Failed to initialize logging: {}", e);
    }

    // initialize the server state
    let layout = StorageLayout::default();
    let result = init_state(layout).await;
    match result {
        Ok(server_state) => {
            // start the server
            server(server_state).await;
        }
        Err(e) => {
            error!("Failed to initialize server state: {}", e);
            std::process::exit(1);
        }
    }
}