// internal crates
use crate::errors::MiruError;
use crate::http::prelude::*;
use crate::services::config_instances::utils;
use crate::services::errors::{
    LatestConfigInstanceNotFound, ServiceErr, ServiceHTTPErr, ServiceStorageErr,
};

use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceCacheKey};
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
    cache: &ConfigInstanceCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<BaseConfigInstance, ServiceErr> {
    // read the latest config instance from the server
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

    // if successful, update the config instance in storage and return it
    let key = ConfigInstanceCacheKey {
        config_slug: args.config_slug().to_string(),
        config_schema_digest: args.config_schema_digest().to_string(),
    };
    if let Some(config_instance) = result {
        let config_instance = utils::convert_cfg_inst_backend_to_storage(
            config_instance,
            args.config_slug().to_string(),
            args.config_schema_digest().to_string(),
        );
        cache
            .write(key, config_instance.clone(), true)
            .await
            .map_err(|e| {
                ServiceErr::StorageErr(ServiceStorageErr {
                    source: e,
                    trace: trace!(),
                })
            })?;
        return Ok(utils::convert_cfg_inst_storage_to_sdk(config_instance));
    }

    // if unsuccessful, attempt to read the latest config instance from storage
    let latest_config_instance = cache.read_optional(key).await.map_err(|e| {
        ServiceErr::StorageErr(ServiceStorageErr {
            source: e,
            trace: trace!(),
        })
    })?;

    match latest_config_instance {
        Some(latest_config_instance) => Ok(utils::convert_cfg_inst_storage_to_sdk(
            latest_config_instance,
        )),
        None => Err(ServiceErr::LatestConfigInstanceNotFound(
            LatestConfigInstanceNotFound {
                config_slug: args.config_slug().to_string(),
                config_schema_digest: args.config_schema_digest().to_string(),
                trace: trace!(),
            },
        )),
    }
}
