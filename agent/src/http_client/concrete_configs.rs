// std
use std::sync::Arc;

// internal crates
use crate::http_client::client::HTTPClient;
use crate::http_client::errors::HTTPErr;
use openapi_client::models::BackendConcreteConfig;
use openapi_client::models::RefreshLatestConcreteConfigRequest;

#[allow(async_fn_in_trait)]
pub trait ConcreteConfigsExt: Send + Sync {
    async fn read_latest_concrete_config(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr>;

    async fn refresh_latest_concrete_config(
        &self,
        request: &RefreshLatestConcreteConfigRequest,
    ) -> Result<BackendConcreteConfig, HTTPErr>;
}

impl ConcreteConfigsExt for HTTPClient {
    async fn read_latest_concrete_config(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr> {
        // build the request
        let url = format!(
            "{}/latest?config_slug={}&config_schema_digest={}",
            self.base_url, config_slug, config_schema_digest
        );
        let request = self.build_get_request(&url, None)?;

        // send the request
        let response = self.send_cached(url, request, self.timeout).await?.0;

        // parse the response
        let cncr_cfg = self
            .parse_json_response_text::<Option<BackendConcreteConfig>>(response)
            .await?;
        Ok(cncr_cfg)
    }

    async fn refresh_latest_concrete_config(
        &self,
        request: &RefreshLatestConcreteConfigRequest,
    ) -> Result<BackendConcreteConfig, HTTPErr> {
        // build the request
        let url = format!("{}/refresh_latest", self.base_url);
        let key = format!(
            "{}:{}:{}",
            url, request.config_slug, request.config_schema_digest,
        );
        let request = self.build_post_request(&url, self.marshal_json_request(request)?, None)?;

        // send the request
        let response = self.send_cached(key, request, self.timeout).await?.0;

        // parse the response
        let response = self
            .parse_json_response_text::<BackendConcreteConfig>(response)
            .await?;
        Ok(response)
    }
}

impl ConcreteConfigsExt for Arc<HTTPClient> {
    async fn read_latest_concrete_config(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr> {
        self.as_ref()
            .read_latest_concrete_config(config_slug, config_schema_digest)
            .await
    }

    async fn refresh_latest_concrete_config(
        &self,
        request: &RefreshLatestConcreteConfigRequest,
    ) -> Result<BackendConcreteConfig, HTTPErr> {
        self.as_ref().refresh_latest_concrete_config(request).await
    }
}
