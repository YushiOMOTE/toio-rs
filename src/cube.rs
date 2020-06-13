use anyhow::{anyhow, Result};
use derive_new::new;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, time::Duration};
use tokio::sync::broadcast;

use crate::{
    ble::{self, PeripheralOps},
    proto::*,
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
    pub async fn go(
        &mut self,
        left: isize,
        right: isize,
        duration: Option<Duration>,
    ) -> Result<()> {
        if left < -100 || left > 100 || right < -100 || right > 100 {
            return Err(anyhow!("Motor speed must be between -100 and 100"));
        }
        let (left_dir, left) = if left > 0 {
            (MotorDir::Forward, left as u8)
        } else {
            (MotorDir::Back, (left * -1) as u8)
        };
        let (right_dir, right) = if right > 0 {
            (MotorDir::Forward, right as u8)
        } else {
            (MotorDir::Back, (right * -1) as u8)
        };

        let buf: Vec<u8> = if let Some(d) = duration {
            let d = d.as_millis();
            if d > 255 {
                return Err(anyhow!("Duration must be less than 256 milliseconds"));
            }
            let d = d.try_into()?;

            Motor::Timed(MotorTimed::new(
                0x01, left_dir, left, 0x02, right_dir, right, d,
            ))
            .try_into()?
        } else {
            Motor::Simple(MotorSimple::new(
                0x01, left_dir, left, 0x02, right_dir, right,
            ))
            .try_into()?
        };

        self.adaptor.write(&UUID_MOTOR, &buf, false).await?;

        Ok(())
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
