// internal crates
use config_agent::http::auth::ClientAuthExt;
use config_agent::http::config_schemas::ConfigSchemasExt;
use config_agent::http::errors::HTTPErr;
use config_agent::http::prelude::*;
use openapi_client::models::{
    ActivateClientRequest,
    BackendConcreteConfig,
    Client,
    HashSchemaSerializedRequest,
    IssueClientTokenRequest,
    IssueClientTokenResponse,
    RefreshLatestConcreteConfigRequest,
    SchemaDigestResponse,
};

// external crates
use async_trait::async_trait;

// ================================== AUTH EXT ===================================== //
pub struct MockAuthClient {
    pub activate_client_result: Box<dyn Fn() -> Result<Client, HTTPErr> + Send + Sync>,
    pub issue_client_token_result: Box<dyn Fn() -> Result<IssueClientTokenResponse, HTTPErr> + Send + Sync>,
}

impl Default for MockAuthClient {
    fn default() -> Self {
        Self {
            activate_client_result: Box::new(|| Ok(Client::default())),
            issue_client_token_result: Box::new(|| Ok(IssueClientTokenResponse::default())),
        }
    }
}

#[async_trait]
impl ClientAuthExt for MockAuthClient {
    async fn activate_client(
        &self,
        _: &str,
        _: &ActivateClientRequest,
    ) -> Result<Client, HTTPErr> {
        (self.activate_client_result)()
    }

    async fn issue_client_token(
        &self,
        _: &str,
        _: &IssueClientTokenRequest,
    ) -> Result<IssueClientTokenResponse, HTTPErr> {
        (self.issue_client_token_result)()
    }
}

// ============================ CONCRETE CONFIGS EXT =============================== //
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

// =========================== CONFIG SCHEMAS EXT ================================== //
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