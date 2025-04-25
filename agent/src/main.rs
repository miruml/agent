
// internal
use config_agent::server::run::run;
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

    // run the server
    let result = run(StorageLayout::default()).await;
    if let Err(e) = result {
        error!("Failed to run the server: {}", e);
    }
}