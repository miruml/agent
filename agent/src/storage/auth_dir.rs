// internal crates
use crate::auth::rsa;
use crate::filesys::{dir::Dir, file::File, path::PathExt};
use crate::storage::{
    errors::StorageErr,
    jwt_file::{JWTFile, JWT},
    prelude::*,
};
use crate::trace;
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

/// Do not initialize this struct directly. Use the `new` method to create a new
/// AuthDir instance.
#[derive(Clone, Debug)]
pub struct AuthDir {
    pub dir: Dir,
    pub private_key_file: File,
    pub public_key_file: File,
    pub jwt_file: JWTFile,
}

impl AuthDir {
    /// Initialize the AuthDir instance at the given Dir instance. This struct holds
    /// information to authorize the device to the server and serves to help create and
    /// retrieve information in the authorization directory. The directory must exist
    /// with valid contents or an error will be thrown.
    pub fn new(dir: Dir) -> Result<Self, StorageErr> {
        // define the files
        let private_key_file = dir.new_file(&Self::private_key_file_name());

        let public_key_file = dir.new_file(&Self::public_key_file_name());

        let jwt_file = JWTFile::create_if_absent(dir.new_file(JWTFile::file_name()))?;

        // validate and return
        let mut auth_dir = AuthDir {
            dir,
            private_key_file,
            public_key_file,
            jwt_file,
        };
        auth_dir.validate()?;
        Ok(auth_dir)
    }

    pub fn name() -> String {
        "auth".to_string()
    }

    pub fn private_key_file_name() -> String {
        "private_key.pem".to_string()
    }

    pub fn public_key_file_name() -> String {
        "public_key.pem".to_string()
    }

    /// Create the authorization directory and its contents. A JSON Web Token file is
    /// created as well as an RSA key pair. An error is thrown if the directory already
    /// exists. pub(super) is used to allow the miru app dir struct to create AuthDir
    /// instance but not expose it to the public API (any code outside of this folder).
    pub fn create(auth_dir: Dir, auth_token: &str) -> Result<Self, StorageErr> {
        // create this directory
        auth_dir
            .assert_doesnt_exist()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        auth_dir
            .create_if_absent()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;

        // create the private and public key files
        let private_key_file = auth_dir.new_file(&Self::private_key_file_name());
        let public_key_file = auth_dir.new_file(&Self::public_key_file_name());
        rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).map_err(|e| {
            StorageErr::AuthErr {
                source: e,
                trace: trace!(),
            }
        })?;

        // Create the JSON Web Token file
        let json_web_token_file = auth_dir.new_file(JWTFile::file_name());
        JWTFile::create(
            json_web_token_file,
            &JWT {
                jwt: auth_token.to_string(),
            },
        )?;

        // validate and return
        AuthDir::new(auth_dir)
    }

    /// Validate the private key file by ensuring it exists. Does not validate the
    /// contents of the private key file since this would needlessly read the contents
    /// into memory (security risk).
    pub fn validate_private_key_file(&self) -> Result<(), StorageErr> {
        // Ensure the private key file exists
        self.private_key_file
            .assert_exists()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(())
    }

    /// Validate the public key file by ensuring it exists. Does not validate the
    /// contents of the public key file since this would needlessly read the contents
    /// into memory (security risk).
    pub fn validate_public_key_file(&self) -> Result<(), StorageErr> {
        // Ensure the public key file exists
        self.public_key_file
            .assert_exists()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(())
    }

    /// Validate the authorization directory by ensuring its contents are valid.
    pub fn validate(&mut self) -> Result<(), StorageErr> {
        self.validate_private_key_file()?;
        self.validate_public_key_file()?;
        if let Err(e) = self.jwt_file.validate() {
            error!(
                "Error validating JWT file: {:?}. JWT will need to be re-issued.",
                e
            );
        }
        Ok(())
    }
}
