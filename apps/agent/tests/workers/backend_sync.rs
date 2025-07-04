// internal crates
use config_agent::workers::backend_sync::handle_listener_event;
use config_agent::mqtt::{
    device::SyncDevice,
    errors::*,
};
use crate::auth::token_mngr::spawn as spawn_token_mngr;

// external crates
use rumqttc::{Event, Incoming, Publish, QoS};



pub mod handle_listener_event {
    use super::*;

    #[tokio::test]
    async fn non_publish_event() {
        let event = Event::Incoming(Incoming::PingReq);
        let is_synced = handle_listener_event(&event).await;
        assert!(is_synced);
    }

    #[tokio::test]
    async fn sync_request_unserializable() {
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            "test".to_string(),
        )));
        let is_synced = handle_listener_event(&event).await;
        assert!(!is_synced);
    }

    #[tokio::test]
    async fn sync_request_is_synced() {
        let payload = SyncDevice {
            is_synced: true,
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let is_synced = handle_listener_event(&event).await;
        assert!(is_synced);
    }

    #[tokio::test]
    async fn sync_request_is_not_synced() {
        let payload = SyncDevice {
            is_synced: false,
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let event = Event::Incoming(Incoming::Publish(Publish::new(
            "test".to_string(),
            QoS::AtLeastOnce,
            payload_bytes,
        )));
        let is_synced = handle_listener_event(&event).await;
        assert!(!is_synced);
    }
}

pub mod handle_listener_error {
    use super::*;

    #[tokio::test]
    async fn authentication_error_triggers_token_refresh() {
        let error = MQTTError::AuthenticationErr(Box::new(AuthenticationErr::InvalidToken));
    }
}

// authentication error triggers token refresh

// other errors are ignored




