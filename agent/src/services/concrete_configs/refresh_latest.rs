// internal crates
use crate::http_client::prelude::*;
use crate::services::errors::ServiceErr;
use crate::storage::concrete_configs::ConcreteConfigCache;
use crate::trace;
use openapi_client::models::BackendConcreteConfig;
use openapi_client::models::RenderLatestConcreteConfigRequest;

pub async fn refresh_latest<T: ConcreteConfigsExt>(
    http_client: &T,
    config_slug: &str,
    config_schema_digest: &str,
    cache: &ConcreteConfigCache,
) -> Result<BackendConcreteConfig, ServiceErr> {
    // this should be retrieved from the agent config
    let client_id = "FIXME";

    // read the latest concrete config from the server
    let payload = RenderLatestConcreteConfigRequest {
        client_id: client_id.to_string(),
        config_slug: config_slug.to_string(),
        config_schema_digest: config_schema_digest.to_string(),
    };
    let cncr_cfg= http_client.refresh_latest_concrete_config(
        &payload
    ).await.map_err(|e| ServiceErr::HTTPErr {
        source: e,
        trace: trace!(),
    })?;

    // update the concrete config in storage
    cache.write(
        config_slug,
        config_schema_digest,
        cncr_cfg.clone(),
        true,
    ).await.map_err(|e| ServiceErr::StorageErr {
        source: e,
        trace: trace!(),
    })?;

    Ok(cncr_cfg)
}