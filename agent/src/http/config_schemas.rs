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
        payload: &HashSchemaSerializedRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr>;
}

impl ConfigSchemasExt for HTTPClient {
    async fn hash_schema(
        &self,
        payload: &HashSchemaSerializedRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        // build the request
        let url = format!("{}/config_schemas/hash/serialized", self.base_url);
        let (request, context) = self.build_post_request(
            &url,
            self.marshal_json_payload(payload)?,
            self.default_timeout,
            None,
        )?;

        // send the request (with caching)
        let key = format!("{}:{}", url, sha256::hash_str(&payload.schema));
        let response = self.send_cached(key, request, &context).await?.0;

        // parse the response
        self
            .parse_json_response_text::<SchemaDigestResponse>(response, &context)
            .await
    }
}

impl ConfigSchemasExt for Arc<HTTPClient> {
    async fn hash_schema(
        &self,
        payload: &HashSchemaSerializedRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        self.as_ref().hash_schema(payload).await
    }
}
