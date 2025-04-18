// internal crates
use crate::http_client::prelude::*;
use crate::errors::MiruError;
use crate::services::errors::ServiceErr;
use crate::services::concrete_configs::utils;
use crate::storage::concrete_configs::ConcreteConfigCache;
use crate::trace;
use openapi_server::models::BaseConcreteConfig;

pub trait ReadLatestArgsI {
    fn config_slug(&self) -> &str;
    fn config_schema_digest(&self) -> &str;
}

pub struct ReadLatestArgs {
    pub config_slug: String,
    pub config_schema_digest: String,
}

impl ReadLatestArgsI for ReadLatestArgs {
    fn config_slug(&self) -> &str { &self.config_slug }
    fn config_schema_digest(&self) -> &str { &self.config_schema_digest }
}

pub async fn read_latest<ArgsT: ReadLatestArgsI, HTTPClientT: ConcreteConfigsExt>(
    args: &ArgsT,
    http_client: &HTTPClientT,
    cache: &ConcreteConfigCache,
) -> Result<BaseConcreteConfig, ServiceErr> {

    // read the latest concrete config from the server
    let result = http_client.read_latest_concrete_config(
        args.config_slug(),
        args.config_schema_digest(),
    ).await;

    // if not a network connection error, return the error (ignore network connection
    // errors)
    let result = match result {
        Ok(result) => result,
        Err(e) => {
            if !e.is_network_connection_error() {
                return Err(ServiceErr::HTTPErr {
                    source: e,
                    trace: trace!(),
                });
            }
            // if is a network connection error, then just return None and continue
            None
        }
    };

    // if successful, update the concrete config in storage and return it
    if let Some(concrete_config) = result {
        let concrete_config = utils::convert_cncr_cfg_backend_to_storage(
            concrete_config,
            args.config_slug().to_string(),
            args.config_schema_digest().to_string(),
        );
        cache.write(
            concrete_config.clone(),
            true,
        ).await.map_err(|e| ServiceErr::StorageErr {
            source: e,
            trace: trace!(),
        })?;
        return Ok(utils::convert_cncr_cfg_storage_to_sdk(concrete_config));
    }

    // if unsuccessful, attempt to read the latest concrete config from storage
    let latest_concrete_config = cache.read_optional(
        args.config_slug(),
        args.config_schema_digest(),
    ).await.map_err(|e| ServiceErr::StorageErr {
        source: e,
        trace: trace!(),
    })?;

    match latest_concrete_config {
        Some(latest_concrete_config) => {
            Ok(utils::convert_cncr_cfg_storage_to_sdk(
                latest_concrete_config,
            ))
        }
        None => {
            Err(ServiceErr::LatestConcreteConfigNotFound {
                config_slug: args.config_slug().to_string(),
                config_schema_digest: args.config_schema_digest().to_string(),
                trace: trace!(),
            })
        }
    }
}