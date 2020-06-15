/// Bluetooth API.
#[macro_use]
pub mod ble;

/// Protocol data structures.
pub mod proto;

mod cube;
mod decode;
mod encode;
mod searcher;

pub use ble::Uuid;
pub use cube::{
    Cube, Event, EventStream, LightOp, LightOps, Note, Sound, SoundOp, SoundOps, SoundPresetId,
};
pub use searcher::*;
