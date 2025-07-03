// standard crates
use std::time::Duration;

// internal crates
use crate::mqtt::errors::*;
use crate::trace;

// external crates
use tokio::time::timeout;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS, Event};
use serde::{Deserialize, Serialize};

// ================================== OPTIONS ====================================== //
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectAddress {
    pub broker: String,
    pub port: u16,
}

impl ConnectAddress {
    pub fn new(broker: String, port: u16) -> Self {
        Self {
            broker,
            port,
        }
    }
}

impl Default for ConnectAddress {
    fn default() -> Self {
        Self {
            broker: "mqtt.miruml.com".to_string(),
            port: 1883
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeouts {
    pub publish: Duration,
    pub subscribe: Duration,
    pub unsubscribe: Duration,
    pub disconnect: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self { 
            publish: Duration::from_secs(3),
            subscribe: Duration::from_secs(3),
            unsubscribe: Duration::from_secs(3),
            disconnect: Duration::from_secs(3),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    pub connect_address: ConnectAddress,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub keep_alive: Duration,
    pub timeouts: Timeouts,
    pub capacity: usize,
}

impl Options {
    pub fn new(
        connect_address: ConnectAddress,
        username: String,
        password: String,
        client_id: String,
        keep_alive: Duration,
        timeouts: Timeouts,
        capacity: usize,
    ) -> Self {
        Self {
            connect_address,
            username,
            password,
            client_id,
            keep_alive,
            timeouts,
            capacity,
        }
    }
}

pub struct OptionsBuilder {
    options: Options,
}

impl OptionsBuilder {
    pub fn new(username: String, password: String, client_id: String) -> Self {
        Self {
            options: Options {
                connect_address: ConnectAddress::default(),
                username,
                password,
                client_id,
                keep_alive: Duration::from_secs(60),
                timeouts: Timeouts::default(),
                capacity: 64,
            },
        }
    }

    pub fn with_connect_address(mut self, connect_address: ConnectAddress) -> Self {
        self.options.connect_address = connect_address;
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.options.username = username;
        self
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.options.password = password;
        self
    }

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.options.client_id = client_id;
        self
    }

    pub fn with_timeouts(mut self, timeouts: Timeouts) -> Self {
        self.options.timeouts = timeouts;
        self
    }

    pub fn build(self) -> Options {
        self.options
    }
}

// =================================== CLIENT ======================================= //
pub struct MQTTClient {
    pub(crate) client: AsyncClient,
    pub(crate) eventloop: EventLoop,
    pub(crate) timeouts: Timeouts,
}

impl MQTTClient {
    pub async fn new(options: Options) -> Self {
        let mut mqtt_options = MqttOptions::new(
            options.client_id,
            options.connect_address.broker,
            options.connect_address.port,
        );
        
        mqtt_options.set_keep_alive(options.keep_alive);
        mqtt_options.set_credentials(options.username, options.password);

        let (client, eventloop) = AsyncClient::new(
            mqtt_options,
            options.capacity
        );

        Self { client, eventloop, timeouts: options.timeouts }
    }

    pub async fn publish(
        &self,
        topic: &str,
        qos: QoS,
        retained: bool,
        payload: &[u8],
    ) -> Result<(), MQTTError> {
        timeout(
            self.timeouts.publish,
            self.client.publish(topic, qos, retained, payload)
        ).await
        .map_err(|_| MQTTError::TimeoutErr(Box::new(TimeoutErr {
            msg: "Publish timeout".to_string(),
            trace: trace!(),
        })))?
        .map_err(|e| MQTTError::PublishErr(Box::new(PublishErr {
            source: e,
            trace: trace!(),
        })))?;

        Ok(())
    }

    pub async fn subscribe(
        &self,
        topic: &str,
        qos: QoS,
    ) -> Result<(), MQTTError> {
        timeout(
            self.timeouts.subscribe,
            self.client.subscribe(topic, qos)
        ).await
        .map_err(|_| MQTTError::TimeoutErr(Box::new(TimeoutErr {
            msg: "Subscribe timeout".to_string(),
            trace: trace!(),
        })))?
        .map_err(|e| MQTTError::PublishErr(Box::new(PublishErr {
            source: e,
            trace: trace!(),
        })))?;

        Ok(())
    }

    pub async fn unsubscribe(
        &self,
        topic: &str,
    ) -> Result<(), MQTTError> {
        timeout(
            self.timeouts.unsubscribe,
            self.client.unsubscribe(topic)
        ).await
        .map_err(|_| MQTTError::TimeoutErr(Box::new(TimeoutErr {
            msg: "Unsubscribe timeout".to_string(),
            trace: trace!(),
        })))?
        .map_err(|e| MQTTError::PublishErr(Box::new(PublishErr {
            source: e,
            trace: trace!(),
        })))?;

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), MQTTError> {
        timeout(
            self.timeouts.disconnect,
            self.client.disconnect()
        ).await
        .map_err(|_| MQTTError::TimeoutErr(Box::new(TimeoutErr {
            msg: "Disconnect timeout".to_string(),
            trace: trace!(),
        })))?
        .map_err(|e| MQTTError::PublishErr(Box::new(PublishErr {
            source: e,
            trace: trace!(),
        })))?;

        Ok(())
    }

    pub async fn poll(&mut self) -> Result<Event, MQTTError> {
        self.eventloop.poll().await.map_err(|e| MQTTError::PollErr(Box::new(PollErr {
            source: e,
            trace: trace!(),
        })))
    }
}