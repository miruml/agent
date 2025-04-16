// internal crates
use crate::http_client::client::HTTPClient;
use crate::services::errors::ServiceErr;
use crate::trace;
use openapi_client::models::BackendConcreteConfig;
use openapi_client::models::RenderLatestConcreteConfigRequest;

pub async fn refresh_latest(
    config_slug: &str,
    config_schema_digest: &str,
) -> Result<BackendConcreteConfig, ServiceErr> {
    // this should be retrieved from the agent config
    let client_id = "FIXME";

    // read the latest concrete config from the server
    let http_client = HTTPClient::new().await;
    let payload = RenderLatestConcreteConfigRequest {
        client_id: client_id.to_string(),
        config_slug: config_slug.to_string(),
        config_schema_digest: config_schema_digest.to_string(),
    };
    http_client.refresh_latest_concrete_config(
        &payload
    ).await.map_err(|e| ServiceErr::HTTPErr {
        source: e,
        trace: trace!(),
    })
}