// internal crates
use config_agent::http::config_schemas::ConfigSchemasExt;
use config_agent::http::errors::HTTPErr;
use config_agent::http::prelude::*;
use openapi_client::models::BackendConcreteConfig;
use openapi_client::models::HashSchemaSerializedRequest;
use openapi_client::models::RefreshLatestConcreteConfigRequest;
use openapi_client::models::SchemaDigestResponse;

pub struct MockConfigSchemasClient {
    pub hash_schema_result: Box<dyn Fn() -> Result<SchemaDigestResponse, HTTPErr> + Send + Sync>,
}

impl Default for MockConfigSchemasClient {
    fn default() -> Self {
        Self {
            hash_schema_result: Box::new(|| Ok(SchemaDigestResponse::default())),
        }
    }
}

impl MockConfigSchemasClient {
    pub fn set_hash_schema<F>(&mut self, hash_schema_result: F)
    where
        F: Fn() -> Result<SchemaDigestResponse, HTTPErr> + Send + Sync + 'static,
    {
        self.hash_schema_result = Box::new(hash_schema_result);
    }
}

impl ConfigSchemasExt for MockConfigSchemasClient {
    async fn hash_schema(
        &self,
        _request: &HashSchemaSerializedRequest,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        (self.hash_schema_result)()
    }
}

pub struct MockConcreteConfigsClient {
    pub read_latest_result:
        Box<dyn Fn() -> Result<Option<BackendConcreteConfig>, HTTPErr> + Send + Sync>,
    pub refresh_latest_result:
        Box<dyn Fn() -> Result<BackendConcreteConfig, HTTPErr> + Send + Sync>,
}

impl Default for MockConcreteConfigsClient {
    fn default() -> Self {
        Self {
            read_latest_result: Box::new(|| Ok(None)),
            refresh_latest_result: Box::new(|| Ok(BackendConcreteConfig::default())),
        }
    }
}

impl MockConcreteConfigsClient {
    pub fn set_read_latest<F>(&mut self, read_latest_result: F)
    where
        F: Fn() -> Result<Option<BackendConcreteConfig>, HTTPErr> + Send + Sync + 'static,
    {
        self.read_latest_result = Box::new(read_latest_result);
    }

    pub fn set_refresh_latest<F>(&mut self, refresh_latest_result: F)
    where
        F: Fn() -> Result<BackendConcreteConfig, HTTPErr> + Send + Sync + 'static,
    {
        self.refresh_latest_result = Box::new(refresh_latest_result);
    }
}

impl ConcreteConfigsExt for MockConcreteConfigsClient {
    async fn read_latest_concrete_config(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<BackendConcreteConfig>, HTTPErr> {
        (self.read_latest_result)()
    }

    async fn refresh_latest_concrete_config(
        &self,
        _: &RefreshLatestConcreteConfigRequest,
    ) -> Result<BackendConcreteConfig, HTTPErr> {
        (self.refresh_latest_result)()
    }
}
