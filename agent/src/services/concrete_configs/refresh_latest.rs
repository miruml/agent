// internal crates
use crate::http::prelude::*;
use crate::services::concrete_configs::utils;
use crate::services::errors::{
    ServiceErr,
    ServiceHTTPErr,
    ServiceStorageErr,
};
use crate::storage::concrete_configs::{ConcreteConfigCache, ConcreteConfigCacheKey};
use crate::trace;
use openapi_client::models::RefreshLatestConcreteConfigRequest;

pub trait RefreshLatestArgsI {
    fn config_slug(&self) -> &str;
    fn config_schema_digest(&self) -> &str;
}

pub struct RefreshLatestArgs {
    pub config_slug: String,
    pub config_schema_digest: String,
}

impl RefreshLatestArgsI for RefreshLatestArgs {
    fn config_slug(&self) -> &str {
        &self.config_slug
    }
    fn config_schema_digest(&self) -> &str {
        &self.config_schema_digest
    }
}

pub async fn refresh_latest<ArgsT: RefreshLatestArgsI, HTTPClientT: ConcreteConfigsExt>(
    args: &ArgsT,
    http_client: &HTTPClientT,
    cache: &ConcreteConfigCache,
) -> Result<openapi_server::models::BaseConcreteConfig, ServiceErr> {
    // this should be retrieved from the agent config
    let client_id = "FIXME";

    // read the latest concrete config from the server
    let payload = RefreshLatestConcreteConfigRequest {
        client_id: client_id.to_string(),
        config_slug: args.config_slug().to_string(),
        config_schema_digest: args.config_schema_digest().to_string(),
    };
    let cncr_cfg = http_client
        .refresh_latest_concrete_config(&payload)
        .await
        .map_err(|e| ServiceErr::HTTPErr(ServiceHTTPErr {
            source: e,
            trace: trace!(),
        }))?;

    // update the concrete config in storage
    let cncr_cfg = utils::convert_cncr_cfg_backend_to_storage(
        cncr_cfg,
        args.config_slug().to_string(),
        args.config_schema_digest().to_string(),
    );
    let key = ConcreteConfigCacheKey {
        config_slug: args.config_slug().to_string(),
        config_schema_digest: args.config_schema_digest().to_string(),
    };
    cache
        .write(key, cncr_cfg.clone(), true)
        .await
        .map_err(|e| ServiceErr::StorageErr(ServiceStorageErr {
            source: e,
            trace: trace!(),
        }))?;

    Ok(utils::convert_cncr_cfg_storage_to_sdk(cncr_cfg))
}
