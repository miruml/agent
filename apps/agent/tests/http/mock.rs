// standard crates
use std::fmt;
use std::sync::{Arc, Mutex};

// internal crates
use config_agent::http::config_instances::{ConfigInstanceFilters, ConfigInstancesExt};
use config_agent::http::config_schemas::{ConfigSchemaFilters, ConfigSchemasExt};
use config_agent::http::devices::DevicesExt;
use config_agent::http::errors::HTTPErr;
use openapi_client::models::{
    ActivateDeviceRequest, ConfigInstance, ConfigInstanceList, ConfigSchema,
    ConfigSchemaList, Device, HashSchemaSerializedRequest, IssueDeviceTokenRequest,
    SchemaDigestResponse, TokenResponse, UpdateConfigInstanceRequest,
};

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

type ListConfigInstancesFn = Box<dyn Fn() -> Result<ConfigInstanceList, HTTPErr> + Send + Sync>;
type ListAllConfigInstancesFn =
    Box<dyn Fn() -> Result<Vec<ConfigInstance>, HTTPErr> + Send + Sync>;
type UpdateConfigInstanceFn = Box<dyn Fn() -> Result<ConfigInstance, HTTPErr> + Send + Sync>;

pub struct MockConfigInstancesClient {
    pub list_config_instances_fn: Arc<Mutex<ListConfigInstancesFn>>,
    pub list_all_config_instances_fn: Arc<Mutex<ListAllConfigInstancesFn>>,
    pub update_config_instance_fn: Arc<Mutex<UpdateConfigInstanceFn>>,
}

impl Default for MockConfigInstancesClient {
    fn default() -> Self {
        Self {
            list_config_instances_fn: Arc::new(Mutex::new(Box::new(|| {
                Ok(ConfigInstanceList::default())
            }))),
            list_all_config_instances_fn: Arc::new(Mutex::new(Box::new(|| Ok(vec![])))),
            update_config_instance_fn: Arc::new(Mutex::new(Box::new(|| {
                Ok(ConfigInstance::default())
            }))),
        }
    }
}

impl MockConfigInstancesClient {
    pub fn set_list_config_instances<F>(&self, list_config_instances_fn: F)
    where
        F: Fn() -> Result<ConfigInstanceList, HTTPErr> + Send + Sync + 'static,
    {
        *self.list_config_instances_fn.lock().unwrap() = Box::new(list_config_instances_fn);
    }

    pub fn set_list_all_config_instances<F>(&self, list_all_config_instances_fn: F)
    where
        F: Fn() -> Result<Vec<ConfigInstance>, HTTPErr> + Send + Sync + 'static,
    {
        *self.list_all_config_instances_fn.lock().unwrap() = Box::new(list_all_config_instances_fn);
    }

    pub fn set_update_config_instance<F>(&self, update_config_instance_fn: F)
    where
        F: Fn() -> Result<ConfigInstance, HTTPErr> + Send + Sync + 'static,
    {
        *self.update_config_instance_fn.lock().unwrap() = Box::new(update_config_instance_fn);
    }
}

impl ConfigInstancesExt for MockConfigInstancesClient {
    async fn list_config_instances(&self, _: &str, _: &str) -> Result<ConfigInstanceList, HTTPErr> {
        (*self.list_config_instances_fn.lock().unwrap())()
    }

    async fn list_all_config_instances<I>(
        &self,
        _: ConfigInstanceFilters,
        _: I,
        _: &str,
    ) -> Result<Vec<ConfigInstance>, HTTPErr>
    where
        I: IntoIterator + Send,
        I::Item: fmt::Display,
    {
        (*self.list_all_config_instances_fn.lock().unwrap())()
    }

    async fn update_config_instance(
        &self,
        _: &str,
        _: &UpdateConfigInstanceRequest,
        _: &str,
    ) -> Result<ConfigInstance, HTTPErr> {
        (*self.update_config_instance_fn.lock().unwrap())()
    }
}

// ============================ CONFIG INSTANCES EXT =============================== //
pub struct HistoryConfigInstancesClient {
    pub update_config_instance_requests: Arc<Mutex<Vec<UpdateConfigInstanceRequest>>>,
}

impl Default for HistoryConfigInstancesClient {
    fn default() -> Self {
        Self {
            update_config_instance_requests: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl HistoryConfigInstancesClient {
    pub fn get_update_config_instance_requests(&self) -> Vec<UpdateConfigInstanceRequest> {
        self.update_config_instance_requests.lock().unwrap().clone()
    }
}

impl ConfigInstancesExt for HistoryConfigInstancesClient {
    async fn list_config_instances(&self, _: &str, _: &str) -> Result<ConfigInstanceList, HTTPErr> {
        Ok(ConfigInstanceList::default())
    }

    async fn list_all_config_instances<I>(
        &self,
        _: ConfigInstanceFilters,
        _: I,
        _: &str,
    ) -> Result<Vec<ConfigInstance>, HTTPErr>
    where
        I: IntoIterator + Send,
        I::Item: fmt::Display,
    {
        Ok(vec![])
    }

    async fn update_config_instance(
        &self,
        _: &str,
        request: &UpdateConfigInstanceRequest,
        _: &str,
    ) -> Result<ConfigInstance, HTTPErr> {
        self.update_config_instance_requests
            .lock()
            .unwrap()
            .push(request.clone());
        Ok(ConfigInstance::default())
    }
}

// =========================== CONFIG SCHEMAS EXT ================================== //
pub struct MockConfigSchemasClient {
    pub hash_schema_fn: Box<dyn Fn() -> Result<SchemaDigestResponse, HTTPErr> + Send + Sync>,
    pub list_config_schemas_fn: Box<dyn Fn() -> Result<ConfigSchemaList, HTTPErr> + Send + Sync>,
    pub find_one_config_schema_fn: Box<dyn Fn() -> Result<ConfigSchema, HTTPErr> + Send + Sync>,
}

impl Default for MockConfigSchemasClient {
    fn default() -> Self {
        Self {
            hash_schema_fn: Box::new(|| Ok(SchemaDigestResponse::default())),
            list_config_schemas_fn: Box::new(|| Ok(ConfigSchemaList::default())),
            find_one_config_schema_fn: Box::new(|| Ok(ConfigSchema::default())),
        }
    }
}

impl MockConfigSchemasClient {
    pub fn set_hash_schema<F>(&mut self, hash_schema_fn: F)
    where
        F: Fn() -> Result<SchemaDigestResponse, HTTPErr> + Send + Sync + 'static,
    {
        self.hash_schema_fn = Box::new(hash_schema_fn);
    }

    pub fn set_list_config_schemas<F>(&mut self, list_config_schemas_fn: F)
    where
        F: Fn() -> Result<ConfigSchemaList, HTTPErr> + Send + Sync + 'static,
    {
        self.list_config_schemas_fn = Box::new(list_config_schemas_fn);
    }

    pub fn set_find_one_config_schema<F>(&mut self, find_one_config_schema_fn: F)
    where
        F: Fn() -> Result<ConfigSchema, HTTPErr> + Send + Sync + 'static,
    {
        self.find_one_config_schema_fn = Box::new(find_one_config_schema_fn);
    }
}

impl ConfigSchemasExt for MockConfigSchemasClient {
    async fn hash_schema(
        &self,
        _request: &HashSchemaSerializedRequest,
        _: &str,
    ) -> Result<SchemaDigestResponse, HTTPErr> {
        (self.hash_schema_fn)()
    }

    async fn list_config_schemas(&self, _: &str, _: &str) -> Result<ConfigSchemaList, HTTPErr> {
        (self.list_config_schemas_fn)()
    }

    async fn find_one_config_schema(
        &self,
        _: ConfigSchemaFilters,
        _: &str,
    ) -> Result<ConfigSchema, HTTPErr> {
        (self.find_one_config_schema_fn)()
    }
}
