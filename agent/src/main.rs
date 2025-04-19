// internal
use config_agent::server::api::server;

#[tokio::main]
async fn main() {
    server().await;
}
