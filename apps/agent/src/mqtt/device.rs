// internal crates
use crate::mqtt::client::MQTTClient;
// use crate::mqtt::device::
use crate::mqtt::errors::MQTTError;

// external crates
use rumqttc::QoS;

pub type SyncDevice = openapi_client::models::SyncDevice;

// =================================== TRAIT ======================================= //
#[allow(async_fn_in_trait)]
pub trait DeviceExt {
    async fn publish_device_sync(&self, device_id: &str) -> Result<(), MQTTError>;
    async fn subscribe_device_sync(&self, device_id: &str) -> Result<(), MQTTError>;
}


impl DeviceExt for MQTTClient {
    async fn publish_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let topic = format!("cmd/devices/{device_id}/sync");
        let payload = SyncDevice { is_synced: true };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        self.publish(&topic, QoS::AtLeastOnce, true, &payload_bytes)
            .await
    }

    async fn subscribe_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let topic = format!("cmd/devices/{device_id}/sync");
        self.subscribe(&topic, QoS::AtLeastOnce).await
    }
}
