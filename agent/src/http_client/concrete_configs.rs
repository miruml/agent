// internal crates
use crate::http_client::errors::HTTPErr;
use crate::http_client::client::HTTPClient;
use openapi_client::models::RenderLatestConcreteConfigRequest;
use openapi_client::models::BackendConcreteConfig;
use openapi_client::models::ConcreteConfigList;

#[allow(async_fn_in_trait)]
pub trait ConcreteConfigsExt: Send + Sync {
    async fn read_latest_concrete_config(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr>;
    async fn refresh_latest_concrete_config(
        &self,
        request: &RenderLatestConcreteConfigRequest,
    ) -> Result<BackendConcreteConfig, HTTPErr>;
}

impl ConcreteConfigsExt for HTTPClient {
    async fn read_latest_concrete_config(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr> {
        // build the request
        let url = format!("{}/concrete_configs?config_slug={}&config_schema_digest={}", self.base_url, config_slug, config_schema_digest);
        let request = self.build_get_request(&url, None)?;

        // send the request
        let response = self.send_cached(
            url,
            request,
            self.timeout,
        ).await?;

        // parse the response
        let cncr_cfg_list = self.parse_json_response_text::<ConcreteConfigList>(response).await?;
        if cncr_cfg_list.data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(cncr_cfg_list.data[0].clone()))
        }
    }

    async fn refresh_latest_concrete_config(
        &self,
        request: &RenderLatestConcreteConfigRequest,
    ) -> Result<BackendConcreteConfig, HTTPErr> {
        // build the request
        let url = format!("{}/render_latest", self.base_url);
        let key = format!(
            "{}:{}:{}",
            url,
            request.config_slug,
            request.config_schema_digest,
        );
        let request = self.build_post_request(
            &url,
            self.marshal_json_request(request)?,
            None,
        )?;

        // send the request
        let response = self.send_cached(
            key,
            request,
            self.timeout,
        ).await?;
        
        // parse the response
        let response = self.parse_json_response_text::<BackendConcreteConfig>(response).await?;
        Ok(response)
    }
}