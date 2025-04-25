// internal
use config_agent::logs::{init, LogLevel};
use config_agent::server::run::{run, RunServerOptions};

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
    let result = run(RunServerOptions::default()).await;
    if let Err(e) = result {
        error!("Failed to run the server: {}", e);
    }
}
