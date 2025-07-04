// standard library
use std::time::Duration;

// internal crates
use crate::errors::{
    DialoguerErr, InstallerCryptErr, InstallerErr, InstallerFileSysErr, InstallerHTTPErr,
    InstallerStorageErr,
};
use crate::{utils, utils::Colors};
use config_agent::crypt::jwt;
use config_agent::http::devices::DevicesExt;
use config_agent::logs::LogLevel;
use config_agent::storage::{agent::Agent, layout::StorageLayout, setup::setup_storage};
use config_agent::trace;
use openapi_client::models::ActivateDeviceRequest;

// external crates
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

const LANDING_PAGE_URL: &str = "https://miruml.com";
const MIRU_DEVICES_PAGE: &str = "https://configs.miruml.com/devices";

type DeviceID = String;

pub struct Installer<HTTPClientT: DevicesExt> {
    layout: StorageLayout,
    http_client: HTTPClientT,
    cur_title: String,
}

impl<HTTPClientT: DevicesExt> Installer<HTTPClientT> {
    pub fn new(layout: StorageLayout, http_client: HTTPClientT) -> Self {
        Installer {
            layout,
            http_client,
            cur_title: "Miru Installation".to_string(),
        }
    }

    // walks user through the installation process
    pub async fn install(&mut self, backend_base_url: &str) -> Result<(), InstallerErr> {
        // setup the storage so that the agent can authenticate its keys and such
        let agent = Agent {
            activated: false,
            ..Default::default()
        };
        setup_storage(&self.layout, &agent).await.map_err(|e| {
            InstallerErr::StorageErr(InstallerStorageErr {
                source: e,
                trace: trace!(),
            })
        })?;

        // authenticate the agent
        let device_id = self.authenticate_agent().await?;

        // update the storage layout to hold the device id and such
        let agent_file = self.layout.agent_file();
        let agent = Agent {
            device_id,
            activated: true,
            backend_base_url: backend_base_url.to_string(),
            log_level: LogLevel::Info,
        };
        agent_file
            .write_json(&agent, true, true)
            .await
            .map_err(|e| {
                InstallerErr::FileSysErr(InstallerFileSysErr {
                    source: e,
                    trace: trace!(),
                })
            })?;

        Ok(())
    }

    pub async fn authenticate_agent(&mut self) -> Result<DeviceID, InstallerErr> {
        loop {
            // grab the jwt token from the user
            let token = self.get_jwt_from_user()?;

            // write the device id to the agent file
            let device_id = jwt::extract_device_id(&token).map_err(|e| {
                InstallerErr::CryptErr(InstallerCryptErr {
                    source: e,
                    trace: trace!(),
                })
            })?;

            // authenticate the device with the server
            let result = self.authenticate_with_server(&device_id, &token).await;
            match result {
                // successful -> return
                Ok(_) => {
                    return Ok(device_id);
                }
                // error -> let the user decide if they want to retry
                Err(e) => {
                    error!("Authentication Error: {:?}", e);
                    utils::print_err_msg(Some(e.to_string()));
                    let retry =
                        utils::confirm("Would you like to retry the authentication process?")?;
                    utils::clear_terminal();
                    if !retry {
                        return Err(e);
                    }
                }
            }
        }
    }

    pub fn get_jwt_from_user(&mut self) -> Result<String, InstallerErr> {
        utils::clear_terminal();
        self.cur_title = "Miru Agent Activation".to_string();
        utils::print_title(&self.cur_title);
        println!(
            "Welcome! {} provides the infrastructure to version, manage, and deploy application configurations at scale. \n",
            utils::format_url(LANDING_PAGE_URL, "Miru")
        );

        println!("To authenticate the miru agent, you'll need to retrieve the authentication token from {} for the device you want to authenticate as.\n", utils::format_url(MIRU_DEVICES_PAGE, MIRU_DEVICES_PAGE));

        // prompt user for their json web token
        let token = Input::with_theme(&utils::input_theme())
            .with_prompt("Enter Authentication Token")
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

    pub async fn authenticate_with_server(
        &mut self,
        device_id: &str,
        token: &str,
    ) -> Result<(), InstallerErr> {
        utils::print_title(&self.cur_title);

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
        let public_key_file = self.layout.auth_dir().public_key_file();
        let public_key_pem = public_key_file.read_string().await.map_err(|e| {
            InstallerErr::FileSysErr(InstallerFileSysErr {
                source: e,
                trace: trace!(),
            })
        })?;
        let payload = ActivateDeviceRequest { public_key_pem };
        let device = self
            .http_client
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
}
