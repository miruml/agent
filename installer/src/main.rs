// standard library
use std::env;

// internal crates
use config_agent::filesys::{dir::Dir, path::PathExt};
use config_agent::http::client::HTTPClient;
use config_agent::logs::{init, LogOptions};
use config_agent::storage::layout::StorageLayout;
use config_agent_installer::installer::Installer;
use config_agent_installer::users::{assert_groupname, assert_username};
use config_agent_installer::utils;
use config_agent::utils::{has_version_flag, version_info};

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[tokio::main]
async fn main() {
    // print the version and exit if that is all the is requested
    if has_version_flag() {
        println!("{}", version_info());
        return;
    }

    match install().await {
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
}

async fn install() -> Result<(), Box<dyn std::error::Error>> {
    // assert the os user and group
    assert_username("miru")?;
    assert_groupname("miru")?;

    // initialize the logger
    let tmp_dir = Dir::create_temp_dir("miru-agent-installer-logs").await?;
    let options = LogOptions {
        // sending logs to stdout will interfere with the installer outputs
        stdout: false,
        log_dir: tmp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let guard = init(options)?;

    // determine the backend url to use for installation
    let default_backend_url = "https://configs.api.miruml.com/internal/agent/v1".to_string();
    let args: Vec<String> = env::args().collect();
    let backend_url = args.get(1).unwrap_or(&default_backend_url);

    // create and run the installer
    let http_client = HTTPClient::new(backend_url).await;
    let mut installer = Installer::new(StorageLayout::default(), http_client);
    installer.install().await?;

    drop(guard);

    Ok(())
}
