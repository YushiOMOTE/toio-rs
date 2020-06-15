# toio-rs

[toio](https://toio.io/) driver in Rust

[![Latest version](https://img.shields.io/crates/v/toio.svg)](https://crates.io/crates/toio)
[![Documentation](https://docs.rs/toio/badge.svg)](https://docs.rs/toio)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Actions Status](https://github.com/YushiOMOTE/toio-rs/workflows/Rust/badge.svg)](https://github.com/YushiOMOTE/toio-rs/actions)

* Supports all the messages defined in [the API reference document](https://toio.github.io/toio.js/).
* Supports async/await.
* Provides the similar capability as [JavaScript version](https://github.com/toio/toio.js/).
* Provides the high-level API, which is easy to use.
* Provides the low-level API, which allows fine-grained control and configuration.
* Plans to be cross-platform. The targets are:
    * macOS
    * Windows 10 (TODO)
    * Linux (TODO)

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
