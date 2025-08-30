
const V1: &str = "v1";

pub fn device_sync_topic(device_id: &str) -> String {
    format!("{V1}/cmd/devices/{device_id}/sync")
}
pub fn device_ping_topic(device_id: &str) -> String {
    format!("{V1}/cmd/devices/{device_id}/ping")
}
pub fn device_pong_topic(device_id: &str) -> String {
    format!("{V1}/resp/devices/{device_id}/pong")
}

pub enum SubscriptionTopics {
    Sync,
    Ping,
    Unknown,
}

pub fn parse_subscription(
    device_id: &str,
    topic: &str
) -> SubscriptionTopics {
    if topic == device_sync_topic(device_id) {
        SubscriptionTopics::Sync
    } else if topic == device_ping_topic(device_id) {
        SubscriptionTopics::Ping
    } else {
        SubscriptionTopics::Unknown
    }
}