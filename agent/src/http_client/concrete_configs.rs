// internal crates
use crate::http_client::errors::HTTPErr;
use crate::http_client::client::HTTPClient;
use openapi_client::models::RenderLatestConcreteConfigRequest;
use openapi_client::models::BackendConcreteConfig;

// external crates
use std::time::Duration;



impl HTTPClient {
    pub(crate) async fn render_latest(
        &self,
        request: &RenderLatestConcreteConfigRequest,
    ) -> Result<BackendConcreteConfig, HTTPErr> {
        let url = format!("{}/render/latest", self.base_url);
        let request = self.build_post_request(url, request, None)?;
    }
}