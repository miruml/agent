// internal crates
use crate::http_client::errors::HTTPErr;
use crate::http_client::client::HTTPClient;
use openapi_client::models::HashSchemaRequest;
use openapi_client::models::SchemaDigestResponse;

// external crates
use std::time::Duration;

impl HTTPClient {
    pub async fn hash_schema(
        &self,
        request: &HashSchemaRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        // build the request
        let url = format!("{}/config_schemas/hash", self.base_url);
        let request = self.build_post_request(
            &url,
            self.marshal_json_request(request)?,
            None,
        )?;

        // send the request
        let response = self.send(
            request,
            Duration::from_secs(10),
        ).await?;

        // parse the response
        let response = self.parse_json_response::<SchemaDigestResponse>(response).await?;
        Ok(response)
    }
}
