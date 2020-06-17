use std::time::Duration;
use toio::Cube;
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for the nearest cube.
    let mut cube = Cube::search().nearest().await.unwrap();

    // Connect.
    cube.connect().await.unwrap();

    // Move forward.
    cube.go(10, 10, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Move backward.
    cube.go(-10, -10, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Spin counterclockwise.
    cube.go(5, 50, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Spin clockwise.
    cube.go(50, 5, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Stop.
    cube.stop().await.unwrap();

    delay_for(Duration::from_secs(1)).await;

    // Spin for 2 seconds.
    cube.go(100, 5, Some(Duration::from_secs(2))).await.unwrap();

    delay_for(Duration::from_secs(3)).await;
}
