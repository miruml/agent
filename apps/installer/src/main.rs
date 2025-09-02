// standard library
use std::collections::HashMap;
use std::env;

// internal crates
use config_agent::filesys::{dir::Dir, path::PathExt};
use config_agent::http::client::HTTPClient;
use config_agent::logs::{init, LogOptions};
use config_agent::storage::layout::StorageLayout;
use config_agent::storage::settings;
use config_agent::utils::{has_version_flag, version_info};
use config_agent_installer::install;
use config_agent_installer::utils;

// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[tokio::main]
async fn main() {
    // print the version and exit if that is all the is requested
    if has_version_flag() {
        println!("{:?}", version_info());
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
    // initialize the logger
    let tmp_dir = Dir::create_temp_dir("miru-agent-installer-logs").await?;
    let options = LogOptions {
        // sending logs to stdout will interfere with the installer outputs
        stdout: false,
        log_dir: tmp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let guard = init(options)?;

    let mut settings = settings::Settings::default();

    let args: Vec<String> = env::args().collect();
    let mut kv_args: HashMap<String, String> = HashMap::new();
    for arg in args.iter().skip(1) {
        if let Some((key, value)) = arg.split_once('=') {
            let clean_key = key.trim_start_matches('-');
            kv_args.insert(clean_key.to_string(), value.to_string());
        }
    }

    // retrieve the activation token
    let token_env_var = "MIRU_ACTIVATION_TOKEN";
    let activation_token = match env::var(token_env_var) {
        Ok(token) => token,
        Err(_) => {
            error!("The {} environment variable is not set", token_env_var);
            return Err(format!("The {} environment variable is not set", token_env_var).into());
        }
    };

    // set optional settings
    if let Some(backend_host) = kv_args.get("backend-host") {
        settings.backend.base_url = format!("{}/agent/v1", backend_host);
    }
    if let Some(mqtt_broker_host) = kv_args.get("mqtt-broker-host") {
        settings.mqtt_broker.host = mqtt_broker_host.to_string();
    }

    // run the installation
    let http_client = HTTPClient::new(&settings.backend.base_url).await;
    let layout = StorageLayout::default();
    install::install(
        &layout,
        &http_client,
        &settings,
        activation_token.as_str(),
        kv_args.get("device-name").map(|name| name.to_string()),
    )
    .await?;

    drop(guard);

    Ok(())
}
