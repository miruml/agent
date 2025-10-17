// standard crates
use std::sync::{Arc, Mutex};

// internal crates
use miru_agent::mqtt::device::DeviceExt;
use miru_agent::mqtt::errors::MQTTError;

#[derive(Clone)]
pub enum MQTTDeviceClientCall {
    PublishDeviceSync(PublishDeviceSyncCall),
    SubscribeDeviceSync(SubscribeDeviceSyncCall),
    SubscribeDevicePing(SubscribeDevicePingCall),
    PublishDevicePong(PublishDevicePongCall),
}

#[derive(Clone)]
pub struct PublishDeviceSyncCall {
    pub device_id: String,
}

#[derive(Clone)]
pub struct SubscribeDeviceSyncCall {
    pub device_id: String,
}

#[derive(Clone)]
pub struct SubscribeDevicePingCall {
    pub device_id: String,
}

#[derive(Clone)]
pub struct PublishDevicePongCall {
    pub device_id: String,
    pub ping_message_id: String,
}

pub struct MockDeviceClient {
    pub publish_device_sync_fn: Box<dyn Fn() -> Result<(), MQTTError> + Send + Sync>,
    pub subscribe_device_sync_fn: Box<dyn Fn() -> Result<(), MQTTError> + Send + Sync>,
    pub subscribe_device_ping_fn: Box<dyn Fn() -> Result<(), MQTTError> + Send + Sync>,
    pub publish_device_pong_fn: Box<dyn Fn() -> Result<(), MQTTError> + Send + Sync>,
    pub calls: Arc<Mutex<Vec<MQTTDeviceClientCall>>>,
}

impl Default for MockDeviceClient {
    fn default() -> Self {
        Self {
            publish_device_sync_fn: Box::new(|| Ok(())),
            subscribe_device_sync_fn: Box::new(|| Ok(())),
            subscribe_device_ping_fn: Box::new(|| Ok(())),
            publish_device_pong_fn: Box::new(|| Ok(())),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl MockDeviceClient {
    pub fn set_publish_device_sync<F>(&mut self, publish_device_sync_fn: F)
    where
        F: Fn() -> Result<(), MQTTError> + Send + Sync + 'static,
    {
        self.publish_device_sync_fn = Box::new(publish_device_sync_fn);
    }

    pub fn set_subscribe_device_sync<F>(&mut self, subscribe_device_sync_fn: F)
    where
        F: Fn() -> Result<(), MQTTError> + Send + Sync + 'static,
    {
        self.subscribe_device_sync_fn = Box::new(subscribe_device_sync_fn);
    }

    pub fn set_subscribe_device_ping<F>(&mut self, subscribe_device_ping_fn: F)
    where
        F: Fn() -> Result<(), MQTTError> + Send + Sync + 'static,
    {
        self.subscribe_device_ping_fn = Box::new(subscribe_device_ping_fn);
    }

    pub fn set_publish_device_pong<F>(&mut self, publish_device_pong_fn: F)
    where
        F: Fn() -> Result<(), MQTTError> + Send + Sync + 'static,
    {
        self.publish_device_pong_fn = Box::new(publish_device_pong_fn);
    }

    pub fn get_calls(&self) -> Vec<MQTTDeviceClientCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn num_publish_device_sync_calls(&self) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| matches!(call, MQTTDeviceClientCall::PublishDeviceSync(_)))
            .count()
    }

    pub fn num_subscribe_device_sync_calls(&self) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| matches!(call, MQTTDeviceClientCall::SubscribeDeviceSync(_)))
            .count()
    }

    pub fn num_subscribe_device_ping_calls(&self) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| matches!(call, MQTTDeviceClientCall::SubscribeDevicePing(_)))
            .count()
    }

    pub fn num_publish_device_pong_calls(&self) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| matches!(call, MQTTDeviceClientCall::PublishDevicePong(_)))
            .count()
    }
}

impl DeviceExt for MockDeviceClient {
    async fn subscribe_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let call = SubscribeDeviceSyncCall {
            device_id: device_id.to_string(),
        };
        self.calls
            .lock()
            .unwrap()
            .push(MQTTDeviceClientCall::SubscribeDeviceSync(call));
        (self.subscribe_device_sync_fn)()
    }

    async fn publish_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let call = PublishDeviceSyncCall {
            device_id: device_id.to_string(),
        };
        self.calls
            .lock()
            .unwrap()
            .push(MQTTDeviceClientCall::PublishDeviceSync(call));
        (self.publish_device_sync_fn)()
    }

    async fn subscribe_device_ping(&self, device_id: &str) -> Result<(), MQTTError> {
        let call = SubscribeDevicePingCall {
            device_id: device_id.to_string(),
        };
        self.calls
            .lock()
            .unwrap()
            .push(MQTTDeviceClientCall::SubscribeDevicePing(call));
        (self.subscribe_device_ping_fn)()
    }

    async fn publish_device_pong(
        &self,
        device_id: &str,
        ping_message_id: String,
    ) -> Result<(), MQTTError> {
        let call = PublishDevicePongCall {
            device_id: device_id.to_string(),
            ping_message_id,
        };
        self.calls
            .lock()
            .unwrap()
            .push(MQTTDeviceClientCall::PublishDevicePong(call));
        (self.publish_device_pong_fn)()
    }
}
