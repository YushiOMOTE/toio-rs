use futures::prelude::*;
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

    let mut events = cube.events().await.unwrap();

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            println!("{:?}", event);
        }
    });

    delay_for(Duration::from_secs(30)).await;
}
