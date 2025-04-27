// standard library
use std::process::{Command, Stdio};
use std::time::Duration;

// internal crates
use crate::errors::{
    InstallerErr,
    InstallerStorageErr,
    InstallerCryptErr,
    InstallerHTTPErr,
    InstallerFileSysErr,
    DialoguerErr,
    IOErr,
    ExecShellErr,
};
use crate::scripts;
use crate::utils;
use config_agent::crypt::jwt;
use config_agent::http::auth::ClientAuthExt;
use config_agent::storage::{
    agent::Agent,
    layout::StorageLayout,
    setup::setup_storage,
};
use config_agent::trace;
use openapi_client::models::ActivateClientRequest;

// external crates
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

const LANDING_PAGE_URL: &str = "https://miruml.com";
const MIRU_AUTH_TOKEN_URL: &str = "https://configs.miruml.com/auth/token";

type ClientID = String;

pub struct Installer<HTTPClientT: ClientAuthExt> {
    layout: StorageLayout,
    http_client: HTTPClientT,
    cur_title: String,
    default_wait_prompt: String,
}

impl<HTTPClientT: ClientAuthExt> Installer<HTTPClientT> {
    pub fn new(
        layout: StorageLayout,
        http_client: HTTPClientT,
    ) -> Self {
        let wait_prompt = "Press Enter to continue".to_string();
        Installer {
            layout,
            http_client,
            cur_title: "Miru Installation".to_string(),
            default_wait_prompt: wait_prompt,
        }
    }

    // walks user through the installation process
    pub async fn install(&mut self) -> Result<(), InstallerErr> {
        // clear the terminal first
        utils::clear_terminal();
        self.intro()?;

        // setup the storage so that the agent can authenticate its keys and such
        let agent = Agent { 
            client_id: "placeholder".to_string(),
            activated: false,
        };
        setup_storage(&self.layout, &agent).await.map_err(|e| InstallerErr::StorageErr(InstallerStorageErr {
            source: e,
            trace: trace!(),
        }))?;

        // authenticate the agent
        let client_id = self.authenticate_agent().await?;

        // update the storage layout to hold the client id and such
        let agent_file = self.layout.agent_file();
        let agent = Agent { client_id, activated: true };
        agent_file.write_json(&agent, true, true).await.map_err(|e| InstallerErr::FileSysErr(InstallerFileSysErr {
            source: e,
            trace: trace!(),
        }))?;

        // post installation script
        let post_installation_script = scripts::post_installation_script();
        self.exec_bash_script(post_installation_script)?;
        utils::clear_terminal();

        // installation complete
        self.cur_title = "Installation Complete".to_string();
        utils::print_title(&self.cur_title);
        println!("Miru has been successfully installed and authenticated on your device!");

        Ok(())
    }

    pub fn intro(&mut self) -> Result<(), InstallerErr> {
        self.cur_title = "Miru Agent Installation".to_string();
        utils::print_title(&self.cur_title);
        println!(
            "Welcome! {} provides the infrastructure to version, manage, and deploy application configurations at scale. \n",
            utils::format_url(LANDING_PAGE_URL, "Miru")
        );
        let prompt = "Press Enter to authenticate the miru agent and deploy your first configuration.\n";
        utils::wait(prompt)?;
        utils::clear_terminal();
        Ok(())
    }

    pub async fn authenticate_agent(&mut self) -> Result<ClientID, InstallerErr> {
        loop {
            // grab the jwt token from the user
            let token = self.get_jwt_from_user()?;

            // write the client id to the agent file
            let client_id = jwt::extract_client_id(&token).map_err(|e| InstallerErr::CryptErr(InstallerCryptErr {
                source: e,
                trace: trace!(),
            }))?;

            // authenticate the device with the server
            let result = self.authenticate_with_server(
                &client_id,
                &token,
            ).await;
            match result {
                // successful -> return
                Ok(_) => {
                    return Ok(client_id);
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
        self.cur_title = "Authenticate the Miru Agent".to_string();
        utils::print_title(&self.cur_title);

        println!("To authenticate the miru agent, you'll need to retrieve the authentication token from {} ({}) for the client you want to authenticate as.\n", utils::format_url(MIRU_AUTH_TOKEN_URL, "Web App"), utils::format_url(MIRU_AUTH_TOKEN_URL, MIRU_AUTH_TOKEN_URL));

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
            .map_err(|e| InstallerErr::DialoguerErr(DialoguerErr {
                source: e,
                trace: trace!(),
            }))?;

        utils::clear_terminal();

        Ok(token)
    }

    pub async fn authenticate_with_server(
        &mut self,
        client_id: &str,
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

        // activate the client with the server
        let public_key_file = self.layout.auth_dir().public_key_file();
        let public_key_pem = public_key_file.read_string().await.map_err(|e| InstallerErr::FileSysErr(InstallerFileSysErr {
            source: e,
            trace: trace!(),
        }))?;
        let payload = ActivateClientRequest {
            public_key_pem,
        };
        let client = self.http_client
            .activate_client(client_id, &payload)
            .await
            .map_err(|e| InstallerErr::HTTPErr(InstallerHTTPErr {
                source: e,
                trace: trace!(),
            }))?;

        // complete
        let msg = format!("Successfully activated the miru agent as the '{}' client\n\n", client.name);
        pb.finish_with_message(msg);
        utils::wait(&self.default_wait_prompt)?;
        utils::clear_terminal();

        Ok(())
    }

    pub fn exec_bash_script(&self, script: &str) -> Result<(), InstallerErr> {
        let mut child = Command::new("bash")
            .args(["-c", script])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| InstallerErr::IOErr(IOErr {
                source: e,
                trace: trace!(),
            }))?;

        let status = child
            .wait()
            .map_err(|e| InstallerErr::IOErr(IOErr {
                source: e,
                trace: trace!(),
            }))?;

        if !status.success() {
            return Err(InstallerErr::ExecShellErr(ExecShellErr {
                msg: "Bash script failed".to_string(),
                trace: trace!(),
            }));
        }

        Ok(())
    }
}
