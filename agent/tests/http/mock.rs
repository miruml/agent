// standard crates
use std::fmt;
use std::sync::{Arc, Mutex};

// internal crates
use miru_agent::http::config_instances::{ConfigInstanceFilters, ConfigInstancesExt};
use miru_agent::http::config_schemas::{ConfigSchemaFilters, ConfigSchemasExt};
use miru_agent::http::devices::DevicesExt;
use miru_agent::http::errors::HTTPErr;
use openapi_client::models::{
    ActivateDeviceRequest, ConfigInstance, ConfigInstanceList, ConfigSchema, ConfigSchemaList,
    Device, HashSchemaSerializedRequest, IssueDeviceTokenRequest, SchemaDigestResponse, UpdateDeviceFromAgentRequest,
    TokenResponse, UpdateConfigInstanceRequest,
};

// ================================ MOCK CLIENT ==================================== //

#[derive(Default)]
pub struct MockClient {
    pub devices_client: MockDevicesClient,
    pub config_instances_client: MockCfgInstsClient,
    pub config_schemas_client: MockCfgSchsClient,
}

impl DevicesExt for MockClient {
    async fn activate_device(
        &self,
        device_id: &str,
        payload: &ActivateDeviceRequest,
        token: &str,
    ) -> Result<Device, HTTPErr> {
        self.devices_client.activate_device(device_id, payload, token).await
    }

    async fn issue_device_token(
        &self,
        device_id: &str,
        payload: &IssueDeviceTokenRequest,
    ) -> Result<TokenResponse, HTTPErr> {
        self.devices_client.issue_device_token(device_id, payload).await
    }

    async fn update_device(
        &self,
        device_id: &str,
        payload: &UpdateDeviceFromAgentRequest,
        token: &str,
    ) -> Result<Device, HTTPErr> {
        self.devices_client.update_device(device_id, payload, token).await
    }
}

impl ConfigInstancesExt for MockClient {
    async fn list_config_instances(
        &self,
        query_params: &str,
        token: &str,
    ) -> Result<ConfigInstanceList, HTTPErr> {
        self.config_instances_client.list_config_instances(query_params, token).await
    }

    async fn list_all_config_instances<I>(
        &self,
        filters: ConfigInstanceFilters,
        expansions: I,
        token: &str,
    ) -> Result<Vec<ConfigInstance>, HTTPErr>
    where
        I: IntoIterator + Send,
        I::Item: fmt::Display,
    {
        self.config_instances_client.list_all_config_instances(filters, expansions, token).await
    }

    async fn update_config_instance(
        &self,
        config_instance_id: &str,
        updates: &UpdateConfigInstanceRequest,
        token: &str,
    ) -> Result<ConfigInstance, HTTPErr> {
        self.config_instances_client.update_config_instance(config_instance_id, updates, token).await
    }
}

impl MockClient {
    pub fn set_list_all_config_instances<F>(&self, list_all_config_instances_fn: F)
    where
        F: Fn() -> Result<Vec<ConfigInstance>, HTTPErr> + Send + Sync + 'static,
    {
        self.config_instances_client.set_list_all_config_instances(list_all_config_instances_fn);
    }

    pub fn set_update_config_instance<F>(&self, update_config_instance_fn: F)
    where
        F: Fn() -> Result<ConfigInstance, HTTPErr> + Send + Sync + 'static,
    {
        self.config_instances_client.set_update_config_instance(update_config_instance_fn);
    }
}

// ================================== DEVICES ====================================== //
#[derive(Clone, Debug, PartialEq)]
pub enum DevicesCall {
    ActivateDevice,
    IssueDeviceToken,
    UpdateDevice,
}

pub struct MockDevicesClient {
    pub activate_device_fn: Box<dyn Fn() -> Result<Device, HTTPErr> + Send + Sync>,
    pub issue_device_token_fn: Box<dyn Fn() -> Result<TokenResponse, HTTPErr> + Send + Sync>,
    pub update_device_fn: Box<dyn Fn() -> Result<Device, HTTPErr> + Send + Sync>,
    pub calls: Arc<Mutex<Vec<DevicesCall>>>,
}

impl Default for MockDevicesClient {
    fn default() -> Self {
        Self {
            activate_device_fn: Box::new(|| Ok(Device::default())),
            issue_device_token_fn: Box::new(|| Ok(TokenResponse::default())),
            update_device_fn: Box::new(|| Ok(Device::default())),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl DevicesExt for MockDevicesClient {
    async fn activate_device(
        &self,
        _: &str,
        _: &ActivateDeviceRequest,
        _: &str,
    ) -> Result<Device, HTTPErr> {
        (self.activate_device_fn)()
    }

    async fn issue_device_token(
        &self,
        _: &str,
        _: &IssueDeviceTokenRequest,
    ) -> Result<TokenResponse, HTTPErr> {
        (self.issue_device_token_fn)()
    }

    async fn update_device(
        &self,
        _: &str,
        _: &UpdateDeviceFromAgentRequest,
        _: &str,
    ) -> Result<Device, HTTPErr> {
        self.calls.lock().unwrap().push(DevicesCall::UpdateDevice);
        (self.update_device_fn)()
    }
}

impl MockDevicesClient {
    pub fn num_update_device_calls(&self) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| **call == DevicesCall::UpdateDevice)
            .count()
    }
}

// ============================== CONFIG INSTANCES ================================= //

type ListConfigInstancesFn = Box<dyn Fn() -> Result<ConfigInstanceList, HTTPErr> + Send + Sync>;
type ListAllConfigInstancesFn = Box<dyn Fn() -> Result<Vec<ConfigInstance>, HTTPErr> + Send + Sync>;
type UpdateConfigInstanceFn = Box<dyn Fn() -> Result<ConfigInstance, HTTPErr> + Send + Sync>;
type UpdateDeviceFn = Box<dyn Fn() -> Result<Device, HTTPErr> + Send + Sync>;

#[derive(Clone, Debug, PartialEq)]
pub enum CfgInstsCall {
    ListConfigInstances,
    ListAllConfigInstances,
    UpdateConfigInstance(UpdateConfigInstanceRequest),
}

pub struct MockCfgInstsClient {
    pub list_config_instances_fn: Arc<Mutex<ListConfigInstancesFn>>,
    pub list_all_config_instances_fn: Arc<Mutex<ListAllConfigInstancesFn>>,
    pub update_config_instance_fn: Arc<Mutex<UpdateConfigInstanceFn>>,
    pub update_device_fn: Arc<Mutex<UpdateDeviceFn>>,

    pub calls: Arc<Mutex<Vec<CfgInstsCall>>>,
}

impl Default for MockCfgInstsClient {
    fn default() -> Self {
        Self {
            list_config_instances_fn: Arc::new(Mutex::new(Box::new(|| {
                Ok(ConfigInstanceList::default())
            }))),
            list_all_config_instances_fn: Arc::new(Mutex::new(Box::new(|| Ok(vec![])))),
            update_config_instance_fn: Arc::new(Mutex::new(Box::new(|| {
                Ok(ConfigInstance::default())
            }))),
            update_device_fn: Arc::new(Mutex::new(Box::new(|| Ok(Device::default())))),

            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl MockCfgInstsClient {
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

    pub fn num_update_config_instance_calls(&self) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| matches!(call, CfgInstsCall::UpdateConfigInstance(_)))
            .count()
    }

    pub fn get_calls(&self) -> Vec<CfgInstsCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl ConfigInstancesExt for MockCfgInstsClient {
    async fn list_config_instances(&self, _: &str, _: &str) -> Result<ConfigInstanceList, HTTPErr> {
        self.calls.lock().unwrap().push(CfgInstsCall::ListConfigInstances);
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
        self.calls.lock().unwrap().push(CfgInstsCall::ListAllConfigInstances);
        (*self.list_all_config_instances_fn.lock().unwrap())()
    }

    async fn update_config_instance(
        &self,
        _: &str,
        request: &UpdateConfigInstanceRequest,
        _: &str,
    ) -> Result<ConfigInstance, HTTPErr> {
        self.calls.lock().unwrap().push(CfgInstsCall::UpdateConfigInstance(request.clone()));
        (*self.update_config_instance_fn.lock().unwrap())()
    }
}

// ============================= CONFIG SCHEMAS ==================================== //
pub struct MockCfgSchsClient {
    pub hash_schema_fn: Box<dyn Fn() -> Result<SchemaDigestResponse, HTTPErr> + Send + Sync>,
    pub list_config_schemas_fn: Box<dyn Fn() -> Result<ConfigSchemaList, HTTPErr> + Send + Sync>,
    pub find_one_config_schema_fn: Box<dyn Fn() -> Result<ConfigSchema, HTTPErr> + Send + Sync>,
}

impl Default for MockCfgSchsClient {
    fn default() -> Self {
        Self {
            hash_schema_fn: Box::new(|| Ok(SchemaDigestResponse::default())),
            list_config_schemas_fn: Box::new(|| Ok(ConfigSchemaList::default())),
            find_one_config_schema_fn: Box::new(|| Ok(ConfigSchema::default())),
        }
    }
}

impl MockCfgSchsClient {
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

impl ConfigSchemasExt for MockCfgSchsClient {
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
