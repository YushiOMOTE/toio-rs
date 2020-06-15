use futures::prelude::*;
use std::time::Duration;
use toio::{proto::*, Cube};
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for the nearest cube.
    let mut cube = Cube::search().nearest().await.unwrap();

    // Connect.
    cube.connect().await.unwrap();

    // Move forward.
    cube.write_msg(
        Message::Motor(Motor::Simple(MotorSimple::new(
            MotorId::Left,
            MotorDir::Forward,
            30,
            MotorId::Right,
            MotorDir::Forward,
            30,
        ))),
        false,
    )
    .await
    .unwrap();

    delay_for(Duration::from_secs(2)).await;

    // Subscribe to raw messages from the cube.
    let mut msgs = cube.raw_msgs().await.unwrap();

    // Send a read request for motor state to the cube.
    cube.read_msg(&UUID_MOTION).await.unwrap();

    // Receive raw messages.
    while let Some(msg) = msgs.next().await {
        match msg {
            Message::Motion(Motion::Detect(d)) => {
                println!("{:?}", d);
                break;
            }
            _ => {}
        }
    }

    delay_for(Duration::from_secs(2)).await;
}
