//! [toio](https://toio.io/) driver in Rust.
//!
//! Supports all the messages defined in [the technical specification](https://toio.github.io/toio-spec/).
//! Provides async/await API.
//! Provides the similar API as [JavaScript version](https://github.com/toio/toio.js/).
//! Also provides the low-level API, which allows fine-grained control and configuration.
//!
//! Plans to be cross-platform. The targets are:
//!
//! * macOS
//! * Windows 10 (TODO)
//! * Linux (TODO)
//!
//! ```no_run
//! use std::time::Duration;
//! use toio::Cube;
//! use tokio::time::delay_for;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Search for the nearest cube.
//!     let mut cube = Cube::search().nearest().await.unwrap();
//!
//!     // Connect.
//!     cube.connect().await.unwrap();
//!
//!     // Print status.
//!     println!("version   : {}", cube.version().await.unwrap());
//!     println!("battery   : {}%", cube.battery().await.unwrap());
//!     println!("button    : {}", cube.button().await.unwrap());
//!
//!     // Move forward.
//!     cube.go(10, 10, None).await.unwrap();
//!
//!     delay_for(Duration::from_secs(3)).await;
//!
//!     // Spin for 2 seconds.
//!     cube.go(100, 5, Some(Duration::from_secs(2))).await.unwrap();
//!
//!     delay_for(Duration::from_secs(3)).await;
//! }
//! ```

/// Abstracts BLE.
#[macro_use]
pub mod ble;

/// Protocol data structures.
pub mod proto;

mod cube;
mod decode;
mod encode;
mod searcher;

pub use cube::{Cube, Event, EventStream, LightOp, SoundOp};
pub use proto::{IdPos, IdStd, Note, Posture, SoundPresetId};
pub use searcher::*;
