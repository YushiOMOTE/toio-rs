use anyhow::Result;
use derive_new::new;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;

use crate::{
    ble::{self, PeripheralOps},
    Searcher,
};

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct LightOp {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub duration: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct Light {
    pub ops: Vec<LightOp>,
    pub repeat: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub enum Note {
    A,
    B,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct SoundOp {
    pub note: Note,
    pub duration: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct SoundOps {
    pub ops: Vec<SoundOp>,
    pub repeat: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub enum Sound {
    Preset(usize),
    Ops(SoundOps),
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct Slope {}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub enum Event {
    Battery(usize),
    Collision(bool),
    Slope(bool),
    Button(bool),
}

pub struct EventStream(broadcast::Receiver<Event>);

pub struct Cube {
    adaptor: ble::Adaptor,
}

impl Cube {
    pub(crate) fn new(adaptor: ble::Adaptor) -> Self {
        Self { adaptor }
    }

    /// Get the searcher.
    pub fn search() -> Searcher {
        Searcher::new()
    }

    /// Get the BLE protocol version.
    pub async fn protocol_version(&mut self) -> Result<String> {
        unimplemented!()
    }

    /// Get the battery status.
    pub async fn battery(&mut self) -> Result<usize> {
        unimplemented!()
    }

    /// Get the collision status.
    pub async fn collision(&mut self) -> Result<bool> {
        unimplemented!()
    }

    /// Get the slope status.
    pub async fn slope(&mut self) -> Result<bool> {
        unimplemented!()
    }

    /// Get the button status.
    pub async fn button(&mut self) -> Result<bool> {
        unimplemented!()
    }

    /// Move the cube.
    pub async fn go(&mut self, _left: isize, _right: isize, _duration: Duration) -> Result<Cube> {
        unimplemented!()
    }

    /// Stop the cube movement.
    pub async fn stop(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Play sound.
    pub async fn play(&mut self, _sound: Sound) -> Result<()> {
        unimplemented!()
    }

    /// Stop playing sound.
    pub async fn stop_sound(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Change the light status.
    pub async fn light(&mut self, _light: Light) -> Result<()> {
        unimplemented!()
    }

    /// Turn on the light.
    pub async fn light_on(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Turn off the light.
    pub async fn light_off(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Connect the cube.
    pub async fn connect(&mut self) -> Result<()> {
        self.adaptor.connect().await?;
        Ok(())
    }

    pub async fn events(&self) -> Result<EventStream> {
        unimplemented!()
    }
}
