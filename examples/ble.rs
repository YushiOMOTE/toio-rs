use futures::prelude::*;
use std::time::Duration;
use toio::{
    ble::{self, PeripheralOps, PeripheralOpsExt},
    proto::*,
};
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for a bluetooth device with the toio service UUID.
    let mut searcher = ble::searcher();
    let mut peripherals = searcher.search(&UUID_SERVICE).await.unwrap();
    let mut peripheral = peripherals.pop().unwrap();

    // Connect to the device.
    peripheral.connect().await.unwrap();

    delay_for(Duration::from_secs(2)).await;

    // Write a message to the device.
    peripheral
        .write_msg(
            Motor::Simple(MotorSimple::new(
                MotorId::Left,
                MotorDir::Forward,
                30,
                MotorId::Right,
                MotorDir::Forward,
                30,
            )),
            false,
        )
        .await
        .unwrap();

    delay_for(Duration::from_secs(2)).await;

    // Subscribe to messages from the device.
    let mut msgs = peripheral.subscribe_msg().unwrap();

    // Send a read request for motor state to the device.
    peripheral.read(&UUID_MOTION).await.unwrap();

    // Receive messages.
    while let Some(msg) = msgs.next().await {
        match msg.unwrap() {
            Message::Motion(Motion::Detect(d)) => {
                println!("{:?}", d);
                break;
            }
            _ => {}
        }
    }

    delay_for(Duration::from_secs(2)).await;
}
