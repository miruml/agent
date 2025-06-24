// internal crates
use crate::http::prelude::*;
use crate::models::config_instance::{
    convert_cfg_inst_backend_to_storage, convert_cfg_inst_storage_to_sdk,
};
use crate::services::errors::{
    ServiceErr,
    ServiceHTTPErr,
    ServiceStorageErr,
};
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceCacheKey};
use crate::trace;
use openapi_client::models::RefreshLatestConfigInstanceRequest;

pub trait RefreshLatestArgsI {
    fn device_id(&self) -> &str;
    fn config_type_slug(&self) -> &str;
    fn config_schema_digest(&self) -> &str;
}

pub struct RefreshLatestArgs {
    pub device_id: String,
    pub config_type_slug: String,
    pub config_schema_digest: String,
}

impl RefreshLatestArgsI for RefreshLatestArgs {
    fn device_id(&self) -> &str {
        &self.device_id
    }
    fn config_type_slug(&self) -> &str {
        &self.config_type_slug
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
        device_id: args.device_id().to_string(),
        config_type_slug: args.config_type_slug().to_string(),
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
    let cfg_inst = convert_cfg_inst_backend_to_storage(
        cfg_inst,
        args.config_type_slug().to_string(),
        args.config_schema_digest().to_string(),
    );
    let key = ConfigInstanceCacheKey {
        config_type_slug: args.config_type_slug().to_string(),
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

    Ok(convert_cfg_inst_storage_to_sdk(cfg_inst))
}
