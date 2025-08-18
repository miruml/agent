// standard library
use std::time::Duration;

// internal crates
use crate::errors::{
    DialoguerErr, InstallerCryptErr, InstallerErr, InstallerFileSysErr, InstallerHTTPErr,
    InstallerStorageErr,
};
use crate::{utils, utils::Colors};
use config_agent::crypt::{jwt, rsa};
use config_agent::filesys::file::File;
use config_agent::http::devices::DevicesExt;
use config_agent::storage::{
    agent::Agent, layout::StorageLayout, settings, setup::clean_storage_setup,
};
use config_agent::trace;
use openapi_client::models::ActivateDeviceRequest;

// external crates
use dialoguer::Password;
use indicatif::{ProgressBar, ProgressStyle};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

const LANDING_PAGE_URL: &str = "https://miruml.com";
const MIRU_DEVICES_PAGE: &str = "https://configs.miruml.com/devices";

type DeviceID = String;

// walks user through the installation process
pub async fn install<HTTPClientT: DevicesExt>(
    layout: &StorageLayout,
    http_client: &HTTPClientT,
    settings: &settings::Settings,
    token: Option<String>,
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
    let device_id = activate_device(http_client, &public_key_file, token, device_name).await?;

    // setup a clean storage layout with the new authentication & device id
    clean_storage_setup(
        layout,
        &Agent {
            device_id,
            activated: true,
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
    provided_token: Option<String>,
    device_name: Option<String>,
) -> Result<DeviceID, InstallerErr> {
    let mut attempt = 0;

    loop {
        // prompt for the jwt token
        let token = if let Some(ref provided_token) = provided_token {
            if attempt == 0 {
                provided_token.clone()
            } else {
                prompt_for_jwt("Miru Agent Activation")?
            }
        } else {
            prompt_for_jwt("Miru Agent Activation")?
        };
        attempt += 1;
        let device_id = jwt::extract_device_id(&token).map_err(|e| {
            InstallerErr::CryptErr(InstallerCryptErr {
                source: e,
                trace: trace!(),
            })
        })?;

        // request the activation from the server
        let result = request_activation(
            http_client,
            public_key_file,
            &token,
            &device_id,
            device_name.clone(),
        )
        .await;
        match result {
            Ok(_) => {
                return Ok(device_id);
            }
            // error -> let user decide if they want to retry
            Err(e) => {
                error!("Activation Error: {:?}", e);
                utils::print_err_msg(Some(e.to_string()));
                let retry = utils::confirm("Would you like to retry the activation process?")?;
                utils::clear_terminal();
                if !retry {
                    return Err(e);
                }
            }
        }
    }
}

pub fn prompt_for_jwt(title: &str) -> Result<String, InstallerErr> {
    utils::clear_terminal();
    utils::print_title(title);
    println!(
        "Welcome! {} provides the infrastructure to version, manage, and deploy application configurations at scale. \n",
        utils::format_url(LANDING_PAGE_URL, "Miru")
    );

    println!("To activate the miru agent, you'll need to retrieve the activation token from {} for the device you want to activate this agent as.\n", utils::format_url(MIRU_DEVICES_PAGE, MIRU_DEVICES_PAGE));

    // prompt for activation token
    let token = Password::with_theme(&utils::input_theme())
        .with_prompt("Enter Activation Token")
        .validate_with(|input: &String| -> Result<(), String> {
            // validate the jwt token
            let result = jwt::validate(input);
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        })
        .interact()
        .map_err(|e| {
            InstallerErr::DialoguerErr(DialoguerErr {
                source: e,
                trace: trace!(),
            })
        })?;

    utils::clear_terminal();

    Ok(token)
}

pub async fn request_activation<HTTPClientT: DevicesExt>(
    http_client: &HTTPClientT,
    public_key_file: &File,
    token: &str,
    device_id: &str,
    device_name: Option<String>,
) -> Result<(), InstallerErr> {

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

    // activate the device with the server
    let public_key_pem = public_key_file.read_string().await.map_err(|e| {
        InstallerErr::FileSysErr(InstallerFileSysErr {
            source: e,
            trace: trace!(),
        })
    })?;
    let payload = ActivateDeviceRequest { 
        public_key_pem,
        name: Some(device_name),
    };
    let device = http_client
        .activate_device(device_id, &payload, token)
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

    Ok(())
}
