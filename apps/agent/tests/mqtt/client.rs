use config_agent::errors::MiruError;
// internal crates
use config_agent::filesys::{dir::Dir, path::PathExt};
use config_agent::logs::{init, LogOptions};
use config_agent::mqtt::client::{ConnectAddress, MQTTClient, OptionsBuilder};

// external crates
use rumqttc::{Event, Incoming};
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

#[ignore]
#[tokio::test]
async fn test_mqtt_client() {
    let dir = Dir::create_temp_dir("mqtt_client_test").await.unwrap();
    let _ = init(LogOptions {
        stdout: true,
        log_dir: dir.path().to_path_buf(),
        ..Default::default()
    });

    let device_id = "dvc_123_test";
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJkZXZpY2UiLCJleHAiOjE3NTE1ODMzOTgsImlhdCI6MTc1MTQ5Njk5OCwiaXNzIjoibWlydSIsInN1YiI6ImR2Y18xMjNfdGVzdCJ9.zK2wU7L7NEuC4zcxAoX58Fi2ozTx-4iU1gV2gQyLI9w";
    let options = OptionsBuilder::new(
        device_id.to_string(),
        token.to_string(),
        device_id.to_string(),
    )
    .with_connect_address(ConnectAddress {
        broker: "dev.mqtt.miruml.com".to_string(),
        port: 1883,
    })
    .build();

    // create the client and subscribe to the device sync topic
    let mut client = MQTTClient::new(options).await;

    client.publish_device_sync(device_id).await.unwrap();

    client.subscribe_device_sync(device_id).await.unwrap();

    // Poll for events
    loop {
        let event = client.poll().await;
        match event {
            Ok(event) => {
                info!("event: {event:?}");
                if let Event::Incoming(Incoming::Publish(publish)) = event {
                    if let Ok(text) = std::str::from_utf8(&publish.payload) {
                        info!("payload as string: {}", text);
                    }
                }
            }
            Err(e) => {
                error!("error: {e:?}");
            }
        }
        time::sleep(Duration::from_secs(2)).await;
    }
}

#[tokio::test]
async fn test_mqtt_client_invalid_broker_url() {
    let username = "test".to_string();
    let password = "test".to_string();
    let options = OptionsBuilder::new(username.clone(), password, username)
        .with_connect_address(ConnectAddress {
            broker: "arglebargle.com".to_string(),
            port: 1883,
        })
        .build();

    // create the client and subscribe to the device sync topic
    let mut client = MQTTClient::new(options).await;

    let event = client.poll().await.unwrap_err();
    assert!(event.is_network_connection_error());
}

#[tokio::test]
async fn test_mqtt_client_invalid_username_or_password() {
    let username = "username".to_string();
    let password = "password".to_string();
    let options = OptionsBuilder::new(username.clone(), password, username)
        .with_connect_address(ConnectAddress {
            broker: "dev.mqtt.miruml.com".to_string(),
            port: 1883,
        })
        .build();

    // create the client and subscribe to the device sync topic
    let mut client = MQTTClient::new(options).await;

    let event = client.poll().await.unwrap_err();
    assert!(!event.is_network_connection_error());
}
