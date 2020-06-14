use anyhow::{anyhow, Context, Result};
use derive_new::new;
use futures::{
    future::{abortable, AbortHandle},
    prelude::*,
    stream::{self, BoxStream},
};
use log::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Mutex, time::timeout};

use crate::{
    ble::{self, PeripheralOps, PeripheralOpsExt},
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
    Version(String),
}

pub type EventStream = BoxStream<'static, Event>;

#[derive(Default, Debug)]
struct Status {
    version: Option<String>,
    battery: Option<usize>,
    collision: Option<bool>,
    slope: Option<bool>,
    button: Option<bool>,
}

macro_rules! fetch_if_none {
    ($self:tt, $field:tt, $msg:tt, { $($t:tt)* }) => {{
        let mut events = $self.events().await?;

        $($t)*

        if $self.status.lock().await.$field.is_none() {
            Ok(timeout(READ_TIMEOUT, async move {
                while let Some(event) = events.next().await {
                    match event {
                        Event::$msg(v) => return Ok(v),
                        _ => {}
                    }
                }
                Err(anyhow!("Stream ends while requesting protocol version"))
            })
            .await.context(format!("Couldn't read {}", stringify!($field)))??)
        } else {
            $self.status
                .lock()
                .await
                .$field
                .clone()
                .ok_or_else(|| anyhow!("Couldn't read {}", stringify!($field)))
        }
    }};
}

pub struct Cube {
    dev: ble::Peripheral,
    status: Arc<Mutex<Status>>,
    handle: Option<AbortHandle>,
}

const READ_TIMEOUT: Duration = Duration::from_secs(2);

impl Cube {
    pub(crate) fn new(dev: ble::Peripheral) -> Self {
        Self {
            dev,
            status: Arc::new(Mutex::new(Status::default())),
            handle: None,
        }
    }

    /// Get the searcher.
    pub fn search() -> Searcher {
        Searcher::new()
    }

    /// Get the BLE protocol version.
    pub async fn version(&mut self) -> Result<String> {
        fetch_if_none!(self, version, Version, {
            self.dev
                .write_msg(
                    &UUID_CONFIG,
                    Config::ProtocolVersion(ProtocolVersion::new()),
                    true,
                )
                .await?;
            self.dev.read(&UUID_CONFIG).await?;
        })
    }

    /// Get the battery status.
    pub async fn battery(&mut self) -> Result<usize> {
        fetch_if_none!(self, battery, Battery, {
            self.dev.read(&UUID_BATTERY).await?;
        })
    }

    /// Get the collision status.
    pub async fn collision(&mut self) -> Result<bool> {
        fetch_if_none!(self, collision, Collision, {
            self.dev.read(&UUID_MOTION).await?;
        })
    }

    /// Get the slope status.
    pub async fn slope(&mut self) -> Result<bool> {
        fetch_if_none!(self, slope, Slope, {
            self.dev.read(&UUID_MOTION).await?;
        })
    }

    /// Get the button status.
    pub async fn button(&mut self) -> Result<bool> {
        fetch_if_none!(self, button, Button, {
            self.dev.read(&UUID_BUTTON).await?;
        })
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
        let adjust = |v: isize| {
            (
                if v > 0 {
                    MotorDir::Forward
                } else {
                    MotorDir::Back
                },
                (v.abs() * 255 / 100) as u8,
            )
        };
        let (left_dir, left) = adjust(left);
        let (right_dir, right) = adjust(right);

        let motor = if let Some(d) = duration {
            let d = d.as_millis();
            if d > 255 {
                return Err(anyhow!("Duration must be less than 256 milliseconds"));
            }
            let d = d as u8;

            Motor::Timed(MotorTimed::new(
                0x01, left_dir, left, 0x02, right_dir, right, d,
            ))
        } else {
            Motor::Simple(MotorSimple::new(
                0x01, left_dir, left, 0x02, right_dir, right,
            ))
        };

        self.dev.write_msg(&UUID_MOTOR, motor, false).await?;

        Ok(())
    }

    /// Stop the cube.
    pub async fn stop(&mut self) -> Result<()> {
        self.go(0, 0, None).await?;
        Ok(())
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
        let status = self.status.clone();
        let mut rx = self.events().await?;
        let (forward, handle) = abortable(async move {
            while let Some(event) = rx.next().await {
                update(&status, event).await
            }
        });
        tokio::spawn(forward);
        self.handle = Some(handle);

        self.dev.connect().await?;

        Ok(())
    }

    pub async fn events(&mut self) -> Result<EventStream> {
        let rx = self.dev.subscribe_msg()?;

        Ok(rx
            .filter_map(move |event| async move {
                match event {
                    Ok(msg) => convert(msg).map(|v| stream::iter(v)),
                    Err(e) => {
                        warn!("Error on handling events: {}", e);
                        None
                    }
                }
            })
            .flatten()
            .boxed())
    }
}

impl Drop for Cube {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.as_ref() {
            handle.abort();
        }
    }
}

async fn update(status: &Arc<Mutex<Status>>, event: Event) {
    let mut status = status.lock().await;
    match event {
        Event::Slope(s) => {
            status.slope = Some(s);
        }
        Event::Collision(c) => {
            status.collision = Some(c);
        }
        Event::Button(b) => {
            status.button = Some(b);
        }
        Event::Battery(b) => {
            status.battery = Some(b);
        }
        Event::Version(b) => {
            status.version = Some(b);
        }
    }
}

fn convert(msg: Message) -> Option<Vec<Event>> {
    match msg {
        Message::Motion(Motion::Detect(m)) => {
            Some(vec![Event::Slope(!m.even), Event::Collision(m.collision)])
        }
        Message::Button(Button::Func(b)) => Some(vec![Event::Button(b == ButtonState::Pressed)]),
        Message::Battery(v) => Some(vec![Event::Battery(v as usize)]),
        Message::Config(Config::ProtocolVersionRes(v)) => Some(vec![Event::Version(
            String::from_utf8_lossy(&v.version).to_string(),
        )]),
        _ => None,
    }
}
