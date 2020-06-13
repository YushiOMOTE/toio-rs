use std::time::Duration;
use toio::Cube;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut searcher = Cube::search();
    let mut cube = searcher.nearest().await.unwrap();
    cube.connect().await.unwrap();
    cube.go(10, 10, None).await.unwrap();
    tokio::time::delay_for(Duration::from_secs(10)).await;
}
