// standard library
use std::time::Duration;

// internal crates
use crate::errors::{
    InstallerCryptErr, InstallerErr, InstallerFileSysErr, InstallerHTTPErr, InstallerStorageErr,
};
use crate::{utils, utils::Colors};
use config_agent::crypt::{jwt, rsa};
use config_agent::filesys::file::File;
use config_agent::http::devices::DevicesExt;
use config_agent::models::device::{Device, DeviceStatus};
use config_agent::storage::{layout::StorageLayout, settings, setup::clean_storage_setup};
use config_agent::utils::version_info;
use config_agent::trace;
use openapi_client::models::ActivateDeviceRequest;

// external crates
use chrono::{DateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// walks user through the installation process
pub async fn install<HTTPClientT: DevicesExt>(
    layout: &StorageLayout,
    http_client: &HTTPClientT,
    settings: &settings::Settings,
    token: &str,
    device_name: Option<String>,
) -> Result<(), InstallerErr> {
    // generate new public and private keys in a temporary directory which will be the
    // device's new authentication if the activation is successful
    let temp_dir = layout.temp_dir();
    let private_key_file = temp_dir.file("private.key");
    let public_key_file = temp_dir.file("public.key");
    rsa::gen_key_pair(4096, &private_key_file, &public_key_file, true)
        .await
        .map_err(|e| {
            InstallerErr::CryptErr(InstallerCryptErr {
                source: e,
                trace: trace!(),
            })
        })?;

    // activate the device
    let device = activate_device(http_client, &public_key_file, token, device_name).await?;

    // setup a clean storage layout with the new authentication & device id
    clean_storage_setup(
        layout,
        &Device {
            id: device.id,
            name: device.name,
            session_id: device.session_id,
            version: version_info().version,
            activated: true,
            status: DeviceStatus::Online,
            last_synced_at: DateTime::<Utc>::UNIX_EPOCH,
            last_connected_at: DateTime::<Utc>::UNIX_EPOCH,
            last_disconnected_at: DateTime::<Utc>::UNIX_EPOCH,
        },
        settings,
        &private_key_file,
        &public_key_file,
    )
    .await
    .map_err(|e| {
        InstallerErr::StorageErr(InstallerStorageErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // delete the temporary directory
    temp_dir.delete().await.map_err(|e| {
        InstallerErr::FileSysErr(InstallerFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;

    Ok(())
}

pub async fn activate_device<HTTPClientT: DevicesExt>(
    http_client: &HTTPClientT,
    public_key_file: &File,
    token: &str,
    device_name: Option<String>,
) -> Result<openapi_client::models::Device, InstallerErr> {
    // progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["|", "/", "-", "\\"])
            .template("{spinner} {msg}")
            .expect("Failed to set template"),
    );
    pb.set_message("Activating Agent with the Miru Cloud...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let device_id = jwt::extract_device_id(token).map_err(|e| {
        InstallerErr::CryptErr(InstallerCryptErr {
            source: e,
            trace: trace!(),
        })
    })?;

    // activate the device with the server
    let public_key_pem = public_key_file.read_string().await.map_err(|e| {
        InstallerErr::FileSysErr(InstallerFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;
    let payload = ActivateDeviceRequest {
        public_key_pem,
        name: device_name,
        agent_version: Some(version_info().version),
    };
    let device = http_client
        .activate_device(&device_id, &payload, token)
        .await
        .map_err(|e| {
            InstallerErr::HTTPErr(InstallerHTTPErr {
                source: e,
                trace: trace!(),
            })
        })?;

    // complete
    let msg = format!(
        "Successfully activated the miru agent as the {} device!\n\n",
        utils::color_text(&device.name, Colors::Green)
    );
    pb.finish_with_message(msg);

    Ok(device)
}
