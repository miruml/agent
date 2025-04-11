// standard library
use std::sync::Arc;
// internal crates
use crate::deserialize_error;
use crate::filesys::{file::File, path::PathExt};
use crate::storage::{
    cached_file::{CachedFile, CachedFilePrivate},
    errors::StorageErr,
};
use crate::trace;
// external crates
use serde::{Deserialize, Deserializer, Serialize};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug, Serialize)]
pub struct JWT {
    pub jwt: String,
}

impl Default for JWT {
    fn default() -> Self {
        Self {
            jwt: "invalidtoken".to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for JWT {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct OptionJWT {
            pub jwt: Option<String>,
        }

        let default = JWT::default();

        let result = OptionJWT::deserialize(deserializer);
        let opt = match result {
            Ok(opt) => opt,
            Err(e) => {
                error!(
                    "Error deserializing JWT: {:?}. Setting to default: '{:?}'",
                    e, default
                );
                return Ok(default);
            }
        };
        Ok(JWT {
            jwt: opt
                .jwt
                .unwrap_or_else(|| deserialize_error!("JWT", "jwt", default.jwt)),
        })
    }
}

#[derive(Clone, Debug)]
pub struct JWTFile {
    file: File,
    // cache the device configuration
    cache: Arc<JWT>,
}

impl JWTFile {
    pub fn create_if_absent(jwt_file: File) -> Result<Self, StorageErr> {
        // create if doesn't exist
        if !jwt_file.exists() {
            jwt_file
                .write_json(&JWT::default(), true)
                .map_err(|e| StorageErr::FileSysErr {
                    source: e,
                    trace: trace!(),
                })?;
        }
        Self::new(jwt_file)
    }
}

impl CachedFilePrivate<JWT> for JWTFile {
    fn set_cache(&mut self, cache: JWT) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<JWT> for JWTFile {
    fn init_struct(jwt_file: File, cache: JWT) -> Self {
        Self {
            file: jwt_file,
            cache: Arc::new(cache),
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    fn file_name() -> &'static str {
        "json_web_token.json"
    }

    fn cache(&self) -> Arc<JWT> {
        self.cache.clone()
    }
}
