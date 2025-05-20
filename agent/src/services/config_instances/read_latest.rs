// internal crates
use crate::errors::MiruError;
use crate::http::prelude::*;
use crate::services::config_instances::utils;
use crate::services::errors::{
    LatestConcreteConfigNotFound, ServiceErr, ServiceHTTPErr, ServiceStorageErr,
};

use crate::storage::config_instances::{ConcreteConfigCache, ConcreteConfigCacheKey};
use crate::trace;
use openapi_server::models::BaseConfigInstance;

// external crates
use serde::Deserialize;

pub trait ReadLatestArgsI {
    fn client_id(&self) -> &str;
    fn config_slug(&self) -> &str;
    fn config_schema_digest(&self) -> &str;
}

#[derive(Debug, Deserialize)]
pub struct ReadLatestArgs {
    pub client_id: String,
    pub config_slug: String,
    pub config_schema_digest: String,
}

impl ReadLatestArgsI for ReadLatestArgs {
    fn client_id(&self) -> &str {
        &self.client_id
    }
    fn config_slug(&self) -> &str {
        &self.config_slug
    }
    fn config_schema_digest(&self) -> &str {
        &self.config_schema_digest
    }
}

pub async fn read_latest<ArgsT: ReadLatestArgsI, HTTPClientT: ConfigInstancesExt>(
    args: &ArgsT,
    cache: &ConcreteConfigCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<BaseConfigInstance, ServiceErr> {
    // read the latest concrete config from the server
    let result = http_client
        .read_latest_config_instance(
            args.client_id(),
            args.config_slug(),
            args.config_schema_digest(),
            token,
        )
        .await;

    // if not a network connection error, return the error (ignore network connection
    // errors)
    let result = match result {
        Ok(result) => result,
        Err(e) => {
            if !e.is_network_connection_error() {
                return Err(ServiceErr::HTTPErr(ServiceHTTPErr {
                    source: e,
                    trace: trace!(),
                }));
            }
            // if is a network connection error, then just return None and continue
            None
        }
    };

    // if successful, update the concrete config in storage and return it
    let key = ConcreteConfigCacheKey {
        config_slug: args.config_slug().to_string(),
        config_schema_digest: args.config_schema_digest().to_string(),
    };
    if let Some(concrete_config) = result {
        let concrete_config = utils::convert_cncr_cfg_backend_to_storage(
            concrete_config,
            args.config_slug().to_string(),
            args.config_schema_digest().to_string(),
        );
        cache
            .write(key, concrete_config.clone(), true)
            .await
            .map_err(|e| {
                ServiceErr::StorageErr(ServiceStorageErr {
                    source: e,
                    trace: trace!(),
                })
            })?;
        return Ok(utils::convert_cncr_cfg_storage_to_sdk(concrete_config));
    }

    // if unsuccessful, attempt to read the latest concrete config from storage
    let latest_concrete_config = cache.read_optional(key).await.map_err(|e| {
        ServiceErr::StorageErr(ServiceStorageErr {
            source: e,
            trace: trace!(),
        })
    })?;

    match latest_concrete_config {
        Some(latest_concrete_config) => Ok(utils::convert_cncr_cfg_storage_to_sdk(
            latest_concrete_config,
        )),
        None => Err(ServiceErr::LatestConcreteConfigNotFound(
            LatestConcreteConfigNotFound {
                config_slug: args.config_slug().to_string(),
                config_schema_digest: args.config_schema_digest().to_string(),
                trace: trace!(),
            },
        )),
    }
}
