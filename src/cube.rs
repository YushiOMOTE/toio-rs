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
    proto::{self, *},
    Searcher,
};

/// A light operation.
#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct LightOp {
    /// The value of red light.
    pub red: u8,
    /// The value of green light.
    pub green: u8,
    /// The value of blue light.
    pub blue: u8,
    /// Duration to turn on the light.
    pub duration: Option<Duration>,
}

/// A sound operation.
#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct SoundOp {
    /// Sound note.
    pub note: Note,
    /// Duration to play sound.
    pub duration: Duration,
}

/// The event sent when the status is updated.
#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub enum Event {
    /// Battery is updated.
    Battery(usize),
    /// Set if the cube collides with an object.
    Collision(bool),
    /// Set if the cube is on a slope.
    Slope(bool),
    /// Set if the button is pressed.
    Button(bool),
    /// Posture of the cube.
    Posture(Posture),
    /// Position id information.
    PosId(Option<IdPos>),
    /// Standard id information.
    StdId(Option<IdStd>),
    /// The protocol version.
    Version(String),
}

/// The stream of events.
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
            Ok(timeout(TIMEOUT, async move {
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

/// The toio cube.
pub struct Cube {
    dev: ble::Peripheral,
    status: Arc<Mutex<Status>>,
    handle: Option<AbortHandle>,
}

const TIMEOUT: Duration = Duration::from_secs(2);

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
                .write_msg(Config::Version(ConfigVersion::new()), true)
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
                    MotorDir::Backward
                },
                (v.abs() * 255 / 100) as u8,
            )
        };
        let (left_dir, left) = adjust(left);
        let (right_dir, right) = adjust(right);

        let motor = if let Some(d) = duration {
            let d = d.as_millis() / 10;
            if d > 255 {
                return Err(anyhow!("Duration must be less than 256 milliseconds"));
            }
            let d = d as u8;

            Motor::Timed(MotorTimed::new(
                MotorId::Left,
                left_dir,
                left,
                MotorId::Right,
                right_dir,
                right,
                d,
            ))
        } else {
            Motor::Simple(MotorSimple::new(
                MotorId::Left,
                left_dir,
                left,
                MotorId::Right,
                right_dir,
                right,
            ))
        };

        self.dev.write_msg(motor, false).await?;

        Ok(())
    }

    /// Stop the cube.
    pub async fn stop(&mut self) -> Result<()> {
        self.go(0, 0, None).await?;
        Ok(())
    }

    /// Play sound preset.
    pub async fn play_preset(&mut self, id: SoundPresetId) -> Result<()> {
        self.dev
            .write_msg(Sound::Preset(SoundPreset::new(id, 255)), true)
            .await?;
        Ok(())
    }

    /// Play sound.
    pub async fn play(&mut self, repeat: usize, ops: Vec<SoundOp>) -> Result<()> {
        if ops.len() == 0 || ops.len() >= 60 {
            return Err(anyhow!("The number of operations must be from 1 to 59"));
        }
        if repeat > 255 {
            return Err(anyhow!("The repeat count must be less than 256"));
        }

        let ops: Result<Vec<_>> = ops
            .iter()
            .map(|op| {
                let d = op.duration.as_millis() / 10;

                if d > 255 {
                    return Err(anyhow!("The duration must be less than 2550 milliseconds"));
                }

                Ok(proto::SoundOp::new(d as u8, op.note, 255))
            })
            .collect();
        let ops = ops?;

        self.dev
            .write_msg(
                Sound::Play(SoundPlay::new(repeat as u8, ops.len() as u8, ops)),
                true,
            )
            .await?;

        Ok(())
    }

    /// Stop playing sound.
    pub async fn stop_sound(&mut self) -> Result<()> {
        self.dev.write_msg(proto::Sound::Stop, true).await?;
        Ok(())
    }

    /// Change the light status.
    pub async fn light(&mut self, repeat: usize, ops: Vec<LightOp>) -> Result<()> {
        if ops.len() == 0 || ops.len() >= 30 {
            return Err(anyhow!("The number of operations must be from 1 to 29"));
        }
        if repeat > 255 {
            return Err(anyhow!("The repeat count must be less than 256"));
        }

        let ops: Result<Vec<_>> = ops
            .iter()
            .map(|op| {
                let d = op
                    .duration
                    .as_ref()
                    .map(|d| d.as_millis() / 10)
                    .unwrap_or(0);

                if d > 255 {
                    return Err(anyhow!("The duration must be less than 2550 milliseconds"));
                }

                Ok(LightOn::new(d as u8, op.red, op.green, op.blue))
            })
            .collect();
        let ops = ops?;

        self.dev
            .write_msg(
                Light::Ctrl(LightCtrl::new(repeat as u8, ops.len() as u8, ops)),
                true,
            )
            .await?;

        Ok(())
    }

    /// Turn on the light.
    pub async fn light_on(
        &mut self,
        red: u8,
        green: u8,
        blue: u8,
        duration: Option<Duration>,
    ) -> Result<()> {
        let duration = duration.as_ref().map(|d| d.as_millis() / 10).unwrap_or(0);

        self.dev
            .write_msg(
                Light::On(LightOn::new(duration as u8, red, green, blue)),
                true,
            )
            .await?;

        Ok(())
    }

    /// Turn off the light.
    pub async fn light_off(&mut self) -> Result<()> {
        self.dev
            .write_msg(Light::Off(LightOff::new()), true)
            .await?;
        Ok(())
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

    /// Subscribe to events.
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
        _ => {}
    }
}

fn convert(msg: Message) -> Option<Vec<Event>> {
    match msg {
        Message::Id(Id::Pos(pos)) => Some(vec![Event::PosId(Some(pos))]),
        Message::Id(Id::Std(std)) => Some(vec![Event::StdId(Some(std))]),
        Message::Id(Id::PosMissed) => Some(vec![Event::PosId(None)]),
        Message::Id(Id::StdMissed) => Some(vec![Event::StdId(None)]),
        Message::Motion(Motion::Detect(m)) => Some(vec![
            Event::Slope(!m.level),
            Event::Collision(m.collision),
            Event::Posture(m.posture),
        ]),
        Message::Button(Button::Func(b)) => Some(vec![Event::Button(b == ButtonState::Pressed)]),
        Message::Battery(v) => Some(vec![Event::Battery(v as usize)]),
        Message::Config(Config::VersionRes(v)) => Some(vec![Event::Version(v.version)]),
        _ => None,
    }
}
