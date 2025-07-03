// internal crates
use crate::mqtt::client::MQTTClient;
use crate::mqtt::errors::MQTTError;

// external crates
use rumqttc::QoS;



impl MQTTClient {
    pub async fn publish_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let topic = format!("cmd/devices/{device_id}/sync/req");
        let payload = "Miru Agent Hello World!";
        self.publish(&topic, QoS::AtLeastOnce, true, payload.as_bytes()).await
    }

    pub async fn subscribe_device_sync(&self, device_id: &str) -> Result<(), MQTTError> {
        let topic = format!("cmd/devices/{device_id}/sync/req");
        self.subscribe(&topic, QoS::AtLeastOnce).await
    }
}
