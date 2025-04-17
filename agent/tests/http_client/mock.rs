// internal crates
use config_agent::http_client::errors::HTTPErr;
use config_agent::http_client::config_schemas::ConfigSchemasExt;
use openapi_client::models::HashSchemaRequest;
use openapi_client::models::SchemaDigestResponse;

#[derive(Default)]
pub struct MockConfigSchemasSuccess {
    pub hash_schema_result: SchemaDigestResponse,
}

impl MockConfigSchemasSuccess {
    pub fn set_hash_schema_result(
        &mut self,
        hash_schema_result: SchemaDigestResponse,
    ) {
        self.hash_schema_result = hash_schema_result;
    }
}

impl ConfigSchemasExt for MockConfigSchemasSuccess {


    async fn hash_schema(
        &self,
        _request: &HashSchemaRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        Ok(self.hash_schema_result.clone())
    }
}


