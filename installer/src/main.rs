// internal crates
use config_agent::logs::{init, LogLevel};
use config_agent::http::client::HTTPClient;
use config_agent::storage::layout::StorageLayout;
use config_agent_installer::installer::Installer;
use config_agent_installer::utils;

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[tokio::main]
async fn main() {

    // initialize the logger
    let results = init(false, LogLevel::Info);
    let guard = match results {
        Ok(guard) => guard,
        Err(e) => {
            println!("Failed to initialize the logger: {:?}", e);
            return;
        }
    };

    // run the installer
    let mut installer = Installer::new(
        StorageLayout::default(),
        HTTPClient::new().await,
    );
    let result = installer.install().await;
    match result {
        Ok(_) => {
            info!("Installation successful");
        }
        Err(e) => {
            error!("Installation failed: {:?}", e);
            utils::clear_terminal();
            utils::print_err_msg(Some(e.to_string()));
            return;
        }
    }

    drop(guard);
}
