# toio-rs

toio driver in Rust

![demo](https://raw.github.com/wiki/YushiOMOTE/toio-rs/demo.gif)

```rust
use std::time::Duration;
use toio::Cube;
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    // Search for the nearest cube
    let mut cube = Cube::search().nearest().await.unwrap();

    // Connect
    cube.connect().await.unwrap();

    // Move forward
    cube.go(20, 20, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Move backward
    cube.go(-15, -15, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Spin counterclockwise
    cube.go(5, 50, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Spin clockwise
    cube.go(50, 5, None).await.unwrap();

    delay_for(Duration::from_secs(3)).await;

    // Stop
    cube.stop().await.unwrap();
}
```
