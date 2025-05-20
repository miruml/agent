// internal crates
use config_agent::http::devices::DevicesExt;
use config_agent::http::config_schemas::ConfigSchemasExt;
use config_agent::http::errors::HTTPErr;
use config_agent::http::prelude::*;
use openapi_client::models::{
    ActivateDeviceRequest, BackendConfigInstance, Device, HashSchemaSerializedRequest,
    IssueDeviceTokenRequest, RefreshLatestConfigInstanceRequest, SchemaDigestResponse,
    TokenResponse,
};

// external crates
use async_trait::async_trait;

// ================================== AUTH EXT ===================================== //
pub struct MockAuthClient {
    pub activate_device_result: Box<dyn Fn() -> Result<Device, HTTPErr> + Send + Sync>,
    pub issue_device_token_result: Box<dyn Fn() -> Result<TokenResponse, HTTPErr> + Send + Sync>,
}

impl Default for MockAuthClient {
    fn default() -> Self {
        Self {
            activate_device_result: Box::new(|| Ok(Device::default())),
            issue_device_token_result: Box::new(|| Ok(TokenResponse::default())),
        }
    }
}

#[async_trait]
impl DevicesExt for MockAuthClient {
    async fn activate_device(
        &self,
        _: &str,
        _: &ActivateDeviceRequest,
        _: &str,
    ) -> Result<Device, HTTPErr> {
        (self.activate_device_result)()
    }

    async fn issue_device_token(
        &self,
        _: &str,
        _: &IssueDeviceTokenRequest,
    ) -> Result<TokenResponse, HTTPErr> {
        (self.issue_device_token_result)()
    }
}

// ============================ CONFIG INSTANCES EXT =============================== //
pub struct MockConfigInstancesClient {
    pub read_latest_result:
        Box<dyn Fn() -> Result<Option<BackendConfigInstance>, HTTPErr> + Send + Sync>,
    pub refresh_latest_result:
        Box<dyn Fn() -> Result<BackendConfigInstance, HTTPErr> + Send + Sync>,
}

impl Default for MockConfigInstancesClient {
    fn default() -> Self {
        Self {
            read_latest_result: Box::new(|| Ok(None)),
            refresh_latest_result: Box::new(|| Ok(BackendConfigInstance::default())),
        }
    }
}

impl MockConfigInstancesClient {
    pub fn set_read_latest<F>(&mut self, read_latest_result: F)
    where
        F: Fn() -> Result<Option<BackendConfigInstance>, HTTPErr> + Send + Sync + 'static,
    {
        self.read_latest_result = Box::new(read_latest_result);
    }

    pub fn set_refresh_latest<F>(&mut self, refresh_latest_result: F)
    where
        F: Fn() -> Result<BackendConfigInstance, HTTPErr> + Send + Sync + 'static,
    {
        self.refresh_latest_result = Box::new(refresh_latest_result);
    }
}

impl ConfigInstancesExt for MockConfigInstancesClient {
    async fn read_latest_config_instance(
        &self,
        _: &str,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<Option<BackendConfigInstance>, HTTPErr> {
        (self.read_latest_result)()
    }

    async fn refresh_latest_config_instance(
        &self,
        _: &RefreshLatestConfigInstanceRequest,
        _: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {
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
        _: &str,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        (self.hash_schema_result)()
    }
}
