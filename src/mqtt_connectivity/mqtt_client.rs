use crate::mqtt_connectivity::handlers::handle_sensor_data;
use crate::types::static_topic::StaticTopic;
use gateway_core::gateway::publisher::Channel;

extern crate paho_mqtt as mqtt;

use std::sync::Arc;
use async_mutex::Mutex;
use futures::{executor::block_on, stream::StreamExt};
use std::time::Duration;
use std::process;

use once_cell::sync::Lazy;

static TOPIC: Lazy<Mutex<StaticTopic>> =
    Lazy::new(|| Mutex::new(StaticTopic::new(String::default())));

///
/// Starts the server on the provided port, the server will hand over requests to the handler functions
///
pub async fn start(
    username: String,
    password: String,
    broker_ip: String,
    broker_port: u16,
    topic: String,
    channel: Arc<Mutex<Channel>>,
) -> () {

    let mut state = State::new(channel).await;

    TOPIC.lock().await.set_topic(topic.clone());

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(format!("{}:{}", broker_ip, broker_port))
        .client_id("rust_async_subscribe")
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        // Get message stream before connecting.
        let mut strm = cli.get_stream(25);

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .clean_session(true)
        .user_name(username)
        .password(password)
        .finalize();

    println!("Connecting to the MQTT server...");
    cli.connect(conn_opts).await?;

    cli.subscribe(TOPIC.lock().await.get_topic(), 1).await?;

    println!("Waiting for messages...");

    // TODO This while loop listens to MQTT and forwards any message to the handle_data function
    // TODO which in turn sends data to the tangle
    while let Some(msg_opt) = strm.next().await {
        if let Some(msg) = msg_opt {
            // println!("{}", msg);
            let payload_str = msg.payload_str();
            state.handle_data(payload_str.to_string()).await;
        }
        else {
            // A "None" means we were disconnected. Try to reconnect...
            println!("Lost connection. Attempting reconnect.");
            while let Err(err) = cli.reconnect().await {
                println!("Error reconnecting: {}", err);
                // For tokio use: tokio::time::delay_for()
                async_std::task::sleep(Duration::from_millis(1000)).await;

                }
            }
        }

    Ok::<(), mqtt::Error>(())
    }
) 
    {
    eprintln!("{}", err);
    }
}

struct State {
    channel: Arc<Mutex<Channel>>,
}

impl State {
    pub async fn new(channel: Arc<Mutex<Channel>>) -> Self {
        Self {
            channel: channel,
        }
    }

    pub async fn handle_data(&mut self, data: String) -> () {
        handle_sensor_data(data, &self.channel).await;
  