// std
use std::sync::Arc;

// internal crates
use crate::http_client::errors::HTTPErr;
use crate::http_client::client::HTTPClient;
use openapi_client::models::HashSchemaRequest;
use openapi_client::models::SchemaDigestResponse;
use crate::utils;

#[allow(async_fn_in_trait)]
pub trait ConfigSchemasExt: Send + Sync {
    async fn hash_schema(
        &self,
        request: &HashSchemaRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr>;
}

impl ConfigSchemasExt for HTTPClient {
    async fn hash_schema(
        &self,
        request: &HashSchemaRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        // build the request
        let url = format!("{}/config_schemas/hash", self.base_url);
        let key = format!(
            "{}:{}",
            url,
            utils::hash_json(&request.schema),
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
        let response = self.parse_json_response_text::<SchemaDigestResponse>(response).await?;
        Ok(response)
    }
}

impl ConfigSchemasExt for Arc<HTTPClient> {
    async fn hash_schema(
        &self,
        request: &HashSchemaRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        self.as_ref().hash_schema(request).await
    }
}