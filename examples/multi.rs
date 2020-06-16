use std::time::Duration;
use toio::Cube;
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for all cubes.
    let mut cubes = Cube::search().all().await.unwrap();

    for (i, cube) in cubes.iter_mut().enumerate() {
        // Connect.
        cube.connect().await.unwrap();

        // Print id.
        println!("Connected {} ({} dBm)", cube.id(), cube.rssi());

        // Move differently.
        if i % 2 == 0 {
            cube.go(100, 0, None).await.unwrap();
        } else {
            cube.go(20, 30, None).await.unwrap();
        }
    }

    delay_for(Duration::from_secs(5)).await;
}
