use std::time::Duration;
use toio::Cube;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut cube = Cube::search().nearest().await.unwrap();

    cube.connect().await.unwrap();
    cube.go(10, 10, None).await.unwrap();
    tokio::time::delay_for(Duration::from_secs(2)).await;
    println!("version: {}", cube.protocol_version().await.unwrap());
    cube.stop().await.unwrap();
    tokio::time::delay_for(Duration::from_secs(2)).await;
    cube.go(20, 20, None).await.unwrap();
    tokio::time::delay_for(Duration::from_secs(2)).await;
    for _ in 0usize..100usize {
        println!("slope: {}", cube.slope().await.unwrap());
        println!("collision: {}", cube.collision().await.unwrap());
        println!("battery: {}", cube.battery().await.unwrap());
        tokio::time::delay_for(Duration::from_secs(1)).await;
    }
    cube.stop().await.unwrap();
}
