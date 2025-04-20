// standard library
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use crate::utils;
use openapi_client::models::HashSchemaRequest;
use openapi_client::models::SchemaDigestResponse;

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
        let key = format!("{}:{}", url, utils::hash_json(&request.schema),);
        let (request, context) = self.build_post_request(
            &url,
            self.marshal_json_request(request)?,
            self.default_timeout,
            None,
        )?;

        // send the request
        let response = self.send_cached(key, request, &context).await?.0;

        // parse the response
        let response = self
            .parse_json_response_text::<SchemaDigestResponse>(response, &context)
            .await?;
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
