// internal crates
use crate::mqtt::client::MQTTClient;
use crate::mqtt::errors::MQTTError;

// external crates
use rumqttc::QoS;

type SyncDevice = openapi_client::models::SyncDevice;

impl MQTTClient {
    pub async fn publish_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let topic = format!("cmd/devices/{device_id}/sync");
        let payload = SyncDevice { is_synced: true };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        self.publish(&topic, QoS::AtLeastOnce, true, &payload_bytes)
            .await
    }

    pub async fn subscribe_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let topic = format!("cmd/devices/{device_id}/sync");
        self.subscribe(&topic, QoS::AtLeastOnce).await
    }
}
