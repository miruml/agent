// internal crates
use crate::http::prelude::*;
use crate::services::config_instances::utils;
use crate::services::errors::{ServiceErr, ServiceHTTPErr, ServiceStorageErr};
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceCacheKey};
use crate::trace;
use openapi_client::models::RefreshLatestConfigInstanceRequest;

pub trait RefreshLatestArgsI {
    fn client_id(&self) -> &str;
    fn config_slug(&self) -> &str;
    fn config_schema_digest(&self) -> &str;
}

pub struct RefreshLatestArgs {
    pub client_id: String,
    pub config_slug: String,
    pub config_schema_digest: String,
}

impl RefreshLatestArgsI for RefreshLatestArgs {
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

pub async fn refresh_latest<ArgsT: RefreshLatestArgsI, HTTPClientT: ConfigInstancesExt>(
    args: &ArgsT,
    cache: &ConfigInstanceCache,
    http_client: &HTTPClientT,
    token: &str,
) -> Result<openapi_server::models::BaseConfigInstance, ServiceErr> {
    // read the latest config instance from the server
    let payload = RefreshLatestConfigInstanceRequest {
        device_id: args.client_id().to_string(),
        config_slug: args.config_slug().to_string(),
        config_schema_digest: args.config_schema_digest().to_string(),
    };
    let cfg_inst = http_client
        .refresh_latest_config_instance(&payload, token)
        .await
        .map_err(|e| {
            ServiceErr::HTTPErr(ServiceHTTPErr {
                source: e,
                trace: trace!(),
            })
        })?;

    // update the config instance in storage
    let cfg_inst = utils::convert_cfg_inst_backend_to_storage(
        cfg_inst,
        args.config_slug().to_string(),
        args.config_schema_digest().to_string(),
    );
    let key = ConfigInstanceCacheKey {
        config_slug: args.config_slug().to_string(),
        config_schema_digest: args.config_schema_digest().to_string(),
    };
    cache
        .write(key, cfg_inst.clone(), true)
        .await
        .map_err(|e| {
            ServiceErr::StorageErr(ServiceStorageErr {
                source: e,
                trace: trace!(),
            })
        })?;

    Ok(utils::convert_cfg_inst_storage_to_sdk(cfg_inst))
}
