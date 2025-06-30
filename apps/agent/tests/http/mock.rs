// standard crates
use std::fmt;

// internal crates
use config_agent::http::config_instances::{ConfigInstanceFilters, ConfigInstancesExt};
use config_agent::http::config_schemas::{ConfigSchemaFilters, ConfigSchemasExt};
use config_agent::http::devices::DevicesExt;
use config_agent::http::errors::HTTPErr;
use openapi_client::models::{
    ActivateDeviceRequest, BackendConfigInstance, ConfigInstanceList, ConfigSchema,
    ConfigSchemaList, Device, HashSchemaSerializedRequest, IssueDeviceTokenRequest,
    SchemaDigestResponse, TokenResponse, UpdateConfigInstanceRequest,
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
    pub list_all_config_instances_result:
        Box<dyn Fn() -> Result<Vec<BackendConfigInstance>, HTTPErr> + Send + Sync>,
    pub update_config_instance_result:
        Box<dyn Fn() -> Result<BackendConfigInstance, HTTPErr> + Send + Sync>,
}

impl Default for MockConfigInstancesClient {
    fn default() -> Self {
        Self {
            list_config_instances_result: Box::new(|| Ok(ConfigInstanceList::default())),
            list_all_config_instances_result: Box::new(|| Ok(vec![])),
            update_config_instance_result: Box::new(|| Ok(BackendConfigInstance::default())),
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

    pub fn set_list_all_config_instances<F>(&mut self, list_all_config_instances_result: F)
    where
        F: Fn() -> Result<Vec<BackendConfigInstance>, HTTPErr> + Send + Sync + 'static,
    {
        self.list_all_config_instances_result = Box::new(list_all_config_instances_result);
    }

    pub fn set_update_config_instance<F>(&mut self, update_config_instance_result: F)
    where
        F: Fn() -> Result<BackendConfigInstance, HTTPErr> + Send + Sync + 'static,
    {
        self.update_config_instance_result = Box::new(update_config_instance_result);
    }
}

#[async_trait]
impl ConfigInstancesExt for MockConfigInstancesClient {
    async fn list_config_instances(
        &self,
        _: &str,
        _: &str,
    ) -> Result<ConfigInstanceList, HTTPErr> {
        (self.list_config_instances_result)()
    }

    async fn list_all_config_instances<I>(
        &self,
        _: ConfigInstanceFilters,
        _: I,
        _: &str,
    ) -> Result<Vec<BackendConfigInstance>, HTTPErr>
    where
        I: IntoIterator + Send,
        I::Item: fmt::Display,
    {
        (self.list_all_config_instances_result)()
    }

    async fn update_config_instance(
        &self,
        _: &str,
        _: &UpdateConfigInstanceRequest,
        _: &str,
    ) -> Result<BackendConfigInstance, HTTPErr> {
        (self.update_config_instance_result)()
    }
}

// =========================== CONFIG SCHEMAS EXT ================================== //
pub struct MockConfigSchemasClient {
    pub hash_schema_result: Box<dyn Fn() -> Result<SchemaDigestResponse, HTTPErr> + Send + Sync>,
    pub list_config_schemas_result:
        Box<dyn Fn() -> Result<ConfigSchemaList, HTTPErr> + Send + Sync>,
    pub find_one_config_schema_result:
        Box<dyn Fn() -> Result<ConfigSchema, HTTPErr> + Send + Sync>,
}

impl Default for MockConfigSchemasClient {
    fn default() -> Self {
        Self {
            hash_schema_result: Box::new(|| Ok(SchemaDigestResponse::default())),
            list_config_schemas_result: Box::new(|| Ok(ConfigSchemaList::default())),
            find_one_config_schema_result: Box::new(|| Ok(ConfigSchema::default())),
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

    pub fn set_find_one_config_schema<F>(&mut self, find_one_config_schema_result: F)
    where
        F: Fn() -> Result<ConfigSchema, HTTPErr> + Send + Sync + 'static,
    {
        self.find_one_config_schema_result = Box::new(find_one_config_schema_result);
    }
}

#[async_trait]
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
        _: &str,
        _: &str,
    ) -> Result<ConfigSchemaList, HTTPErr> {
        (self.list_config_schemas_result)()
    }

    async fn find_one_config_schema(
        &self,
        _: ConfigSchemaFilters,
        _: &str,
    ) -> Result<ConfigSchema, HTTPErr> {
        (self.find_one_config_schema_result)()
    }
}
