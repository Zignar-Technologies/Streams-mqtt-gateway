use gateway_core::gateway::publisher::Channel;
use local::mqtt_connectivity::mqtt_client;
use local::types::config::Config;

use std::fs::File;
use std::sync::Arc;
use async_mutex::Mutex;

#[tokio::main]
async fn main() -> () {
    //read configuration file
    let config: Config = serde_json::from_reader(File::open("config.json").unwrap()).unwrap();

    println!("Starting....");

    let channel = Arc::new(Mutex::new(Channel::new(
        &config.node,
        config.local_pow,
        None,
    ).await));
    let (addr, msg) = match channel.lock().await.open().await {
        Ok(a) => a,
        Err(_) => panic!("Could not connect to IOTA Node, try with another node!"),
    };
    println!("Channel root: {:?}", format!("{}:{}", addr, msg));
    println!(
        "\n To read the messages copy the channel root into https://explorer.iot2tangle.io/ \n "
    );

    mqtt_client::start(
        config.username,
        config.password,
        config.broker_ip,
        config.broker_port,
        config.topic,
        channel,
    )
    .await
}
