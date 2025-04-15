// internal crates
use crate::http_client::errors::HTTPErr;
use crate::http_client::client::HTTPClient;
use crate::openapi::RenderLatestRequest;
use crate::openapi::RenderLatestResponse;

// external crates
use std::time::Duration;



impl HTTPClient {
    pub(crate) async fn render_latest(
        &self,
        request: &RenderLatestRequest,
    ) -> Result<RenderLatestResponse, HTTPErr> {
        let url = format!("{}/render/latest", self.base_url);
        let request = self.build_post_request(url, request, None)?;
    }
}