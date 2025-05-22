// standard library
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use openapi_client::models::BackendConfigInstance;
use openapi_client::models::RefreshLatestConfigInstanceRequest;

#[allow(async_fn_in_trait)]
pub trait ConfigInstancesExt: Send + Sync {
    async fn read_latest_config_instance(
        &self,
        device_id: &str,
        config_type_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConfigInstance>, HTTPErr>;

    async fn refresh_latest_config_instance(
        &self,
        payload: &RefreshLatestConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr>;
}

impl HTTPClient {
    fn config_instances_url(&self) -> String {
        format!("{}/config_instances", self.base_url)
    }
}

impl ConfigInstancesExt for HTTPClient {
    async fn read_latest_config_instance(
        &self,
        device_id: &str,
        config_type_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConfigInstance>, HTTPErr> {
        // build the request
        let url = format!(
            "{}/latest?device_id={}&config_type_slug={}&config_schema_digest={}",
            self.config_instances_url(),
            device_id,
            config_type_slug,
            config_schema_digest
        );
        let (request, context) = self.build_get_request(&url, self.default_timeout, Some(token))?;

        // send the request (with caching)
        let response = self.send_cached(url, request, &context).await?.0;

        // parse the response
        self.parse_json_response_text::<Option<BackendConfigInstance>>(response, &context)
            .await
    }

    async fn refresh_latest_config_instance(
        &self,
        payload: &RefreshLatestConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {
        // build the request
        let url = format!("{}/refresh_latest", self.config_instances_url());
        let key = format!(
            "{}:{}:{}",
            url, payload.config_type_slug, payload.config_schema_digest,
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
        self.parse_json_response_text::<BackendConfigInstance>(response, &context)
            .await
    }
}

impl ConfigInstancesExt for Arc<HTTPClient> {
    async fn read_latest_config_instance(
        &self,
        device_id: &str,
        config_type_slug: &str,
        config_schema_digest: &str,
        token: &str,
    ) -> Result<Option<BackendConfigInstance>, HTTPErr> {
        self.as_ref()
            .read_latest_config_instance(device_id, config_type_slug, config_schema_digest, token)
            .await
    }

    async fn refresh_latest_config_instance(
        &self,
        request: &RefreshLatestConfigInstanceRequest,
        token: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {
        self.as_ref()
            .refresh_latest_config_instance(request, token)
            .await
    }
}
