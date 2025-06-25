// internal crates
use config_agent::http::config_schemas::ConfigSchemasExt;
use config_agent::http::devices::DevicesExt;
use config_agent::http::errors::HTTPErr;
use config_agent::http::prelude::*;
use openapi_client::models::{
    ActivateDeviceRequest, ConfigInstanceActivityStatus, ConfigInstanceErrorStatus,
    ConfigInstanceList, ConfigInstanceTargetStatus, ConfigSchemaList, Device,
    HashSchemaSerializedRequest, IssueDeviceTokenRequest, SchemaDigestResponse, TokenResponse,
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
    pub list_config_instances_result:
        Box<dyn Fn() -> Result<ConfigInstanceList, HTTPErr> + Send + Sync>,
}

impl Default for MockConfigInstancesClient {
    fn default() -> Self {
        Self {
            list_config_instances_result: Box::new(|| Ok(ConfigInstanceList::default())),
        }
    }
}

impl MockConfigInstancesClient {
    pub fn set_list_config_instances<F>(&mut self, list_config_instances_result: F)
    where
        F: Fn() -> Result<ConfigInstanceList, HTTPErr> + Send + Sync + 'static,
    {
        self.list_config_instances_result = Box::new(list_config_instances_result);
    }
}

impl ConfigInstancesExt for MockConfigInstancesClient {
    async fn list_config_instances(
        &self,
        _: String,
        _: &[String],
        _: &[ConfigInstanceTargetStatus],
        _: &[ConfigInstanceActivityStatus],
        _: &[ConfigInstanceErrorStatus],
        _: &str,
    ) -> Result<ConfigInstanceList, HTTPErr> {
        (self.list_config_instances_result)()
    }
}

// =========================== CONFIG SCHEMAS EXT ================================== //
pub struct MockConfigSchemasClient {
    pub hash_schema_result: Box<dyn Fn() -> Result<SchemaDigestResponse, HTTPErr> + Send + Sync>,
    pub list_config_schemas_result:
        Box<dyn Fn() -> Result<ConfigSchemaList, HTTPErr> + Send + Sync>,
}

impl Default for MockConfigSchemasClient {
    fn default() -> Self {
        Self {
            hash_schema_result: Box::new(|| Ok(SchemaDigestResponse::default())),
            list_config_schemas_result: Box::new(|| Ok(ConfigSchemaList::default())),
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

    pub fn set_list_config_schemas<F>(&mut self, list_config_schemas_result: F)
    where
        F: Fn() -> Result<ConfigSchemaList, HTTPErr> + Send + Sync + 'static,
    {
        self.list_config_schemas_result = Box::new(list_config_schemas_result);
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

    async fn list_config_schemas(
        &self,
        _: &[String],
        _: &[String],
        _: &str,
    ) -> Result<ConfigSchemaList, HTTPErr> {
        (self.list_config_schemas_result)()
    }
}
