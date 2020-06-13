use std::time::Duration;
use toio::Cube;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut cube = Cube::search().nearest().await.unwrap();

    cube.connect().await.unwrap();
    cube.go(10, 10, None).await.unwrap();
    tokio::time::delay_for(Duration::from_secs(2)).await;
    cube.stop().await.unwrap();
    tokio::time::delay_for(Duration::from_secs(2)).await;
    cube.go(30, 30, None).await.unwrap();
    tokio::time::delay_for(Duration::from_secs(2)).await;
    cube.stop().await.unwrap();
}
