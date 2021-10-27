use serde_json::Value;
use crate::timestamp_in_sec;
use std::sync::Arc;
use async_mutex::Mutex;

use gateway_core::gateway::publisher::Channel;

///
/// Handles the request from the sensor by parsing the provieded data into the SensorData Format.
/// It authenticates the device through the "device" attribute, and if successfull published the data to the Tangle
/// through the streams channel
///
pub async fn handle_sensor_data(
    data: String,
    channel: &Arc<Mutex<Channel>>,
) -> () {
    let data = data.to_owned();
    //let clean_str = std::str::from_utf8(&data).unwrap().replace("\u{0}", "");
    let json_data: serde_json::Result<Value> = serde_json::from_str(&data);
    match json_data {
        Ok(sensor_data) => {
                let mut channel = channel.lock().await;
                match channel.write_signed(&sensor_data).await {
                    Ok(msg_id) => println!("{:?}", msg_id),
                    Err(_e) => {
                        println!("Error: Could not send data to Tangle, try switching nodes");
                        ()
                    }
                };
        }
        Err(_e) => {
            println!(
                "New Message Recieved -- {:?} -- incorrectly formatted Data",
                timestamp_in_sec()
            );
        }
    }
    ()
}