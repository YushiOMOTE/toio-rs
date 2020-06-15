use std::time::Duration;
use toio::{Cube, LightOp};
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for the nearest cube.
    let mut cube = Cube::search().nearest().await.unwrap();

    // Connect.
    cube.connect().await.unwrap();

    // Turn on.
    cube.light_on(255, 255, 255, None).await.unwrap();

    delay_for(Duration::from_secs(2)).await;

    // Turn off.
    cube.light_off().await.unwrap();

    delay_for(Duration::from_secs(2)).await;

    // Light as programmed.
    cube.light(
        10,
        vec![
            LightOp::new(255, 0, 0, Some(Duration::from_millis(100))),
            LightOp::new(0, 255, 0, Some(Duration::from_millis(100))),
            LightOp::new(0, 0, 255, Some(Duration::from_millis(100))),
        ],
    )
    .await
    .unwrap();

    delay_for(Duration::from_secs(4)).await;
}
