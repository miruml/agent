// standard library
use std::sync::Arc;

// internal crates
use crate::http::client::HTTPClient;
use crate::http::errors::HTTPErr;
use crate::crypt::sha256;
use openapi_client::models::hash_schema_serialized_request::HashSchemaSerializedRequest;
use openapi_client::models::SchemaDigestResponse;

#[allow(async_fn_in_trait)]
pub trait ConfigSchemasExt: Send + Sync {
    async fn hash_schema(
        &self,
        request: &HashSchemaSerializedRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr>;
}

impl ConfigSchemasExt for HTTPClient {
    async fn hash_schema(
        &self,
        request: &HashSchemaSerializedRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        // build the request
        let url = format!("{}/config_schemas/hash/serialized", self.base_url);
        let key = format!("{}:{}", url, sha256::hash_str(&request.schema));
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
        request: &HashSchemaSerializedRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        self.as_ref().hash_schema(request).await
    }
}
