use config_agent::errors::MiruError;
// internal crates
use config_agent::filesys::{dir::Dir, path::PathExt};
use config_agent::logs::{init, LogOptions};
use config_agent::mqtt::client::{
    ConnectAddress,
    Credentials,
    MQTTClient,
    OptionsBuilder,
    poll,
};

// external crates
use rumqttc::{Event, Incoming, QoS};
use tracing::{error, info};

#[tokio::test]
async fn test_mqtt_client() {
    let dir = Dir::create_temp_dir("mqtt_client_test").await.unwrap();
    let _ = init(LogOptions {
        stdout: true,
        log_dir: dir.path().to_path_buf(),
        ..Default::default()
    });

    let username = "username";
    let password = "password";
    let options = OptionsBuilder::new(
        Credentials::new(username.to_string(), password.to_string()),
    )
    .with_connect_address(ConnectAddress {
        broker: "broker.emqx.io".to_string(),
        port: 1883,
    })
    .build();

    // create the client and subscribe to the device sync topic
    let (client, mut eventloop) = MQTTClient::new(&options).await;

    let topic = "a/unique/topic/string/for/miru";

    client.subscribe(topic, QoS::AtLeastOnce).await.unwrap();

    let payload = "test";
    client.publish(topic, QoS::AtLeastOnce, false, payload.as_bytes()).await.unwrap();


    // read the published message
    let event = poll(&mut eventloop).await;
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

    client.unsubscribe(topic).await.unwrap();

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn invalid_broker_url() {
    let credentials = Credentials::new(
        "test".to_string(),
        "test".to_string(),
    );
    let options = OptionsBuilder::new(credentials)
        .with_connect_address(ConnectAddress {
            broker: "arglebargle.com".to_string(),
            port: 1883,
        })
        .build();

    // create the client and subscribe to the device sync topic
    let (_, mut eventloop) = MQTTClient::new(&options).await;

    let event = poll(&mut eventloop).await.unwrap_err();
    assert!(event.is_network_connection_error());
}

#[tokio::test]
async fn invalid_username_or_password() {
    let credentials = Credentials::new(
        "username".to_string(),
        "password".to_string(),
    );
    let options = OptionsBuilder::new(credentials)
        .with_connect_address(ConnectAddress {
            broker: "dev.mqtt.miruml.com".to_string(),
            port: 8883,
        })
        .build();

    // create the client and subscribe to the device sync topic
    let (_, mut eventloop) = MQTTClient::new(&options).await;

    let event = poll(&mut eventloop).await.unwrap_err();
    assert!(!event.is_network_connection_error());
}
