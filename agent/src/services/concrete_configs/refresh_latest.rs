// internal crates
use crate::http_client::prelude::*;
use crate::services::errors::ServiceErr;
use crate::storage::layout::StorageLayout;
use crate::storage::concrete_configs::LatestConcreteConfigRegistry;
use crate::trace;
use openapi_client::models::BackendConcreteConfig;
use openapi_client::models::RenderLatestConcreteConfigRequest;

pub async fn refresh_latest<T: ConcreteConfigsExt>(
    http_client: &T,
    config_slug: &str,
    config_schema_digest: &str,
) -> Result<BackendConcreteConfig, ServiceErr> {
    // this should be retrieved from the agent config
    let client_id = "FIXME";

    // read the latest concrete config from the server
    let payload = RenderLatestConcreteConfigRequest {
        client_id: client_id.to_string(),
        config_slug: config_slug.to_string(),
        config_schema_digest: config_schema_digest.to_string(),
    };
    let result = http_client.refresh_latest_concrete_config(
        &payload
    ).await.map_err(|e| ServiceErr::HTTPErr {
        source: e,
        trace: trace!(),
    })?;

    // update the concrete config in storage
    let storage_layout = StorageLayout::new_default();
    let latest_cncr_cfg_reg = LatestConcreteConfigRegistry::new(
        storage_layout.latest_cncr_cfg_registry(),
    );
    latest_cncr_cfg_reg.insert(
        config_slug,
        config_schema_digest,
        &result,
        true,
    ).await.map_err(|e| ServiceErr::StorageErr {
        source: e,
        trace: trace!(),
    })?;

    Ok(result)
}