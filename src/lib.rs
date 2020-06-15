/// Bluetooth API.
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
