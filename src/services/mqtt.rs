use embedded_svc::mqtt::client::{Client, Connection, MessageImpl, Publish, QoS};
use embedded_svc::utils::mqtt::client::ConnState;
use esp_idf_svc::mqtt::client::*;
use esp_idf_sys::EspError;
use anyhow::Result;
use log::*;
use std::thread;

pub struct Config {
  client_id: &'static str,
  url: &'static str,
}

static config: Config = Config {
  client_id: "rust-esp32-std-demo",
  url: "mqtts://broker.emqx.io:8883",
};

pub fn start() -> Result<EspMqttClient<ConnState<MessageImpl, EspError>>> {
  info!("About to start MQTT client");

  let conf = MqttClientConfiguration {
      client_id: Some(config.client_id),
      crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

      ..Default::default()
  };

  let (mut client, mut connection) =
      EspMqttClient::new_with_conn(config.url, &conf)?;

  info!("MQTT client started");

  // Need to immediately start pumping the connection for messages, or else subscribe() and publish() below will not work
  // Note that when using the alternative constructor - `EspMqttClient::new` - you don't need to
  // spawn a new thread, as the messages will be pumped with a backpressure into the callback you provide.
  // Yet, you still need to efficiently process each message in the callback without blocking for too long.
  //
  // Note also that if you go to http://tools.emqx.io/ and then connect and send a message to topic
  // "rust-esp32-std-demo", the client configured here should receive it.
  thread::spawn(move || {
      info!("MQTT Listening for messages");

      while let Some(msg) = connection.next() {
          match msg {
              Err(e) => info!("MQTT Message ERROR: {}", e),
              Ok(msg) => info!("MQTT Message: {:?}", msg),
          }
      }

      info!("MQTT connection loop exit");
  });

  client.subscribe("rust-esp32-std-demo", QoS::AtMostOnce)?;

  info!("Subscribed to all topics (rust-esp32-std-demo)");

  client.publish(
      "rust-esp32-std-demo",
      QoS::AtMostOnce,
      false,
      "Hello from rust-esp32-std-demo!".as_bytes(),
  )?;

  info!("Published a hello message to topic \"rust-esp32-std-demo\"");

  Ok(client)
}