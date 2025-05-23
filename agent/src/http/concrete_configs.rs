// standard library
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use openapi_client::models::BackendConcreteConfig;
use openapi_client::models::RefreshLatestConcreteConfigRequest;

#[allow(async_fn_in_trait)]
pub trait ConcreteConfigsExt: Send + Sync {
    async fn read_latest_concrete_config(
        &self,
        client_id: &str,
        config_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr>;

    async fn refresh_latest_concrete_config(
        &self,
        payload: &RefreshLatestConcreteConfigRequest,
        token: &str,
    ) -> Result<BackendConcreteConfig, HTTPErr>;
}

impl HTTPClient {
    fn concrete_configs_url(&self) -> String {
        format!("{}/concrete_configs", self.base_url)
    }
}

impl ConcreteConfigsExt for HTTPClient {
    async fn read_latest_concrete_config(
        &self,
        client_id: &str,
        config_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr> {
        // build the request
        let url = format!(
            "{}/latest?client_id={}&config_slug={}&config_schema_digest={}",
            self.concrete_configs_url(),
            client_id,
            config_slug,
            config_schema_digest
        );
        let (request, context) = self.build_get_request(&url, self.default_timeout, Some(token))?;

        // send the request (with caching)
        let response = self.send_cached(url, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<Option<BackendConcreteConfig>>(response, &context)
            .await
    }

    async fn refresh_latest_concrete_config(
        &self,
        payload: &RefreshLatestConcreteConfigRequest,
        token: &str,
    ) -> Result<BackendConcreteConfig, HTTPErr> {
        // build the request
        let url = format!("{}/refresh_latest", self.concrete_configs_url());
        let key = format!(
            "{}:{}:{}",
            url, payload.config_slug, payload.config_schema_digest,
        );
        let (request, context) = self.build_post_request(
            &url,
            self.marshal_json_payload(payload)?,
            self.default_timeout,
            Some(token),
        )?;

        // send the request
        let response = self.send_cached(key, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<BackendConcreteConfig>(response, &context)
            .await
    }
}

impl ConcreteConfigsExt for Arc<HTTPClient> {
    async fn read_latest_concrete_config(
        &self,
        client_id: &str,
        config_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr> {
        self.as_ref()
            .read_latest_concrete_config(client_id, config_slug, config_schema_digest, token)
            .await
    }

    async fn refresh_latest_concrete_config(
        &self,
        request: &RefreshLatestConcreteConfigRequest,
        token: &str,
    ) -> Result<BackendConcreteConfig, HTTPErr> {
        self.as_ref()
            .refresh_latest_concrete_config(request, token)
            .await
    }
}
