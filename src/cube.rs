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
    ble::{self, PeripheralOps, PeripheralOpsExt, Uuid},
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
    /// Position information.
    Position(Option<Position>),
    /// Standard id information.
    StdId(Option<StdId>),
    /// The protocol version.
    Version(String),
}

/// The stream of events.
pub type EventStream = BoxStream<'static, Event>;

/// The stream of raw messages.
pub type MessageStream = BoxStream<'static, Message>;

/// The standard id information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, new)]
pub struct StdId {
    pub id: u32,
    pub angle: u16,
}

impl From<IdStd> for StdId {
    fn from(p: IdStd) -> Self {
        Self::new(p.value, p.angle)
    }
}

/// The cube position information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, new)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub angle: u16,
}

impl From<IdPos> for Position {
    fn from(p: IdPos) -> Self {
        Self::new(p.cube_x, p.cube_y, p.cube_angle)
    }
}

#[derive(Default, Debug)]
struct Status {
    version: Option<String>,
    battery: Option<usize>,
    collision: Option<bool>,
    slope: Option<bool>,
    button: Option<bool>,
    position: Option<Option<Position>>,
    std_id: Option<Option<StdId>>,
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

/// The cube.
///
/// Provides API to control the cube. The API has two types:
///
/// * High-level API
/// * Low-level API
///
/// The high-level API provides easy-to-use basic feature to control the cube such as
/// moving, turning on/off the light, and playing sound.
///
/// The low-level API provides the API for more fine-grained control and configuration.
/// The API allows to send/receive all the raw protocol messages,
/// which allows to use all the features of toio cube defined in the specification.
///
/// This is the example of the high-level API:
///
/// ```no_run
/// use std::time::Duration;
/// use toio::Cube;
/// use tokio::time::delay_for;
///
/// #[tokio::main]
/// async fn main() {
///     // Search for the nearest cube.
///     let mut cube = Cube::search().nearest().await.unwrap();
///
///     // Connect.
///     cube.connect().await.unwrap();
///
///     // Print status.
///     println!("version   : {}", cube.version().await.unwrap());
///     println!("battery   : {}%", cube.battery().await.unwrap());
///     println!("button    : {}", cube.button().await.unwrap());
///
///     // Move forward.
///     cube.go(10, 10, None).await.unwrap();
///
///     delay_for(Duration::from_secs(3)).await;
///
///     // Spin for 2 seconds.
///     cube.go(100, 5, Some(Duration::from_secs(2))).await.unwrap();
///
///     delay_for(Duration::from_secs(3)).await;
/// }
/// ```
pub struct Cube {
    dev: ble::Peripheral,
    status: Arc<Mutex<Status>>,
    handle: Option<AbortHandle>,
}

const READ_TIMEOUT: Duration = Duration::from_secs(5);

impl Cube {
    pub(crate) fn new(dev: ble::Peripheral) -> Self {
        Self {
            dev,
            status: Arc::new(Mutex::new(Status::default())),
            handle: None,
        }
    }

    /// Returns [`Searcher`][] instance to search for cubes.
    ///
    /// To find the nearest cube,
    ///
    /// ```no_run
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///
    ///     cube.connect().await.unwrap();
    /// }
    /// ```
    ///
    /// To find all cubes,
    ///
    /// ```no_run
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cubes = Cube::search().all().await.unwrap();
    ///
    ///     for mut cube in cubes {
    ///         cube.connect().await.unwrap();
    ///     }
    /// }
    /// ```
    ///
    /// By default, the search timeout is 3 seconds. Use [`Cube::search_timeout`][]
    /// to set custom timeout.
    pub fn search() -> Searcher {
        Searcher::new()
    }

    /// Gets the device id.
    pub fn id(&self) -> &str {
        self.dev.id()
    }

    /// Gets the signal strength.
    pub fn rssi(&self) -> i32 {
        self.dev.rssi()
    }

    /// Gets the BLE protocol version.
    pub async fn version(&mut self) -> Result<String> {
        fetch_if_none!(self, version, Version, {
            self.dev
                .write_msg(Config::Version(ConfigVersion::new()), true)
                .await?;
            self.dev.read(&UUID_CONFIG).await?;
        })
    }

    /// Gets the battery status.
    ///
    /// Returns the percentage of the remaining battery.
    pub async fn battery(&mut self) -> Result<usize> {
        fetch_if_none!(self, battery, Battery, {
            self.dev.read(&UUID_BATTERY).await?;
        })
    }

    /// Gets the collision status.
    ///
    /// Returns `true` if the cube is in collision.
    pub async fn collision(&mut self) -> Result<bool> {
        fetch_if_none!(self, collision, Collision, {
            self.dev.read(&UUID_MOTION).await?;
        })
    }

    /// Gets the slope status.
    ///
    /// Returns `true` if the cube slopes.
    pub async fn slope(&mut self) -> Result<bool> {
        fetch_if_none!(self, slope, Slope, {
            self.dev.read(&UUID_MOTION).await?;
        })
    }

    /// Gets the button status.
    ///
    /// Returns `true` if the button is pressed.
    pub async fn button(&mut self) -> Result<bool> {
        fetch_if_none!(self, button, Button, {
            self.dev.read(&UUID_BUTTON).await?;
        })
    }

    /// Gets the position information.
    ///
    /// Returns the position information which is read by the sensor.
    /// Returns `None` if no position information is available.
    pub async fn position(&mut self) -> Result<Option<Position>> {
        fetch_if_none!(self, position, Position, {
            self.dev.read(&UUID_ID).await?;
        })
    }

    /// Gets the standard id.
    ///
    /// Returns the standard id which is read by the sensor.
    /// Returns `None` if no id is available.
    pub async fn std_id(&mut self) -> Result<Option<StdId>> {
        fetch_if_none!(self, std_id, StdId, {
            self.dev.read(&UUID_ID).await?;
        })
    }

    /// Moves the cube.
    ///
    /// `left` and `right` are the rotation speed of each wheel.
    /// The value must be in the range from -100 to 100.
    /// The negative number rotates backward, while the positive rotates forward.
    /// If specified, the wheels rotate for the given duration. If the duration is `None`,
    /// wheels rotate forever.
    /// The duration must be in the range from 1 to 2559 milliseconds.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Move forward.
    ///     cube.go(10, 10, None).await.unwrap();
    ///
    ///     // Move backward.
    ///     cube.go(-10, -10, None).await.unwrap();
    ///
    ///     // Spin counterclockwise.
    ///     cube.go(5, 50, None).await.unwrap();
    ///
    ///     // Spin clockwise for 1 second.
    ///     cube.go(50, 5, Some(Duration::from_secs(1))).await.unwrap();
    /// }
    /// ```
    pub async fn go(
        &mut self,
        left: isize,
        right: isize,
        duration: Option<Duration>,
    ) -> Result<()> {
        if left < -100 || left > 100 || right < -100 || right > 100 {
            return Err(anyhow!("Wheel speed must be between -100 and 100"));
        }
        let adjust = |v: isize| {
            (
                if v > 0 {
                    MotorDir::Forward
                } else {
                    MotorDir::Backward
                },
                (v.abs() * (115 - 7) / 100 + 7) as u8,
            )
        };
        let (left_dir, left) = adjust(left);
        let (right_dir, right) = adjust(right);

        let motor = if let Some(d) = duration {
            let d = d.as_millis() / 10;
            if d > 255 {
                return Err(anyhow!("Duration must be less than 2560 milliseconds"));
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

    /// Stops the cube.
    ///
    /// Both wheels stop rotating.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use tokio::time::delay_for;
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Move forward.
    ///     cube.go(10, 10, None).await.unwrap();
    ///
    ///     delay_for(Duration::from_secs(3)).await;
    ///
    ///     // Stop the cube.
    ///     cube.stop().await.unwrap();
    /// }
    /// ```
    pub async fn stop(&mut self) -> Result<()> {
        self.go(0, 0, None).await?;
        Ok(())
    }

    /// Plays sound preset.
    ///
    /// ```no_run
    /// use toio::{Cube, SoundPresetId};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = toio::Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     cube.play_preset(SoundPresetId::Enter).await.unwrap();
    /// }
    /// ```
    pub async fn play_preset(&mut self, id: SoundPresetId) -> Result<()> {
        self.dev
            .write_msg(Sound::Preset(SoundPreset::new(id, 255)), true)
            .await?;
        Ok(())
    }

    /// Plays sound.
    ///
    /// Play sound in accordance with the list of sound operations.
    /// The number of sound operations must be less than 60.
    /// The duration for each sound must be in the range from 1 to 2559 milliseconds.
    /// The repeat count must be less than 256.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use toio::{Cube, SoundOp, Note};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = toio::Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     cube.play(
    ///         // Repeats three times.
    ///         3,
    ///         // Plays two sound for 500 milliseconds for each.
    ///         vec![
    ///             SoundOp::new(Note::C5, Duration::from_millis(500)),
    ///             SoundOp::new(Note::A6, Duration::from_millis(500)),
    ///         ],
    ///     ).await.unwrap();
    /// }
    /// ```
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
                let d = (op.duration.as_millis() / 10).max(1);

                if d > 255 {
                    return Err(anyhow!("The duration must be less than 2560 milliseconds"));
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

    /// Stops playing sound.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use tokio::time::delay_for;
    /// use toio::{Cube, SoundOp, Note};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = toio::Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Starts playing sound.
    ///     cube.play(1, vec![SoundOp::new(Note::C5, Duration::from_secs(2))]).await.unwrap();
    ///
    ///     delay_for(Duration::from_secs(1)).await;
    ///
    ///     // Stops the sound.
    ///     cube.stop_sound().await.unwrap();
    /// }
    /// ```
    pub async fn stop_sound(&mut self) -> Result<()> {
        self.dev.write_msg(proto::Sound::Stop, true).await?;
        Ok(())
    }

    /// Turns on the light as programmed.
    ///
    /// The light color is set by RGB value, each of which must be in range 0 to 255.
    /// The number of light operations must be less than 30.
    /// The repeat count must be less than 256.
    /// The duration of each light operation must be less than 2560 milliseconds.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use toio::{Cube, LightOp};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = toio::Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     cube.light(
    ///         // Repeats 10 times.
    ///         10,
    ///         // Turns on the red, green, blue light for 100 milliseconds for each.
    ///         vec![
    ///             LightOp::new(255, 0, 0, Some(Duration::from_millis(100))),
    ///             LightOp::new(0, 255, 0, Some(Duration::from_millis(100))),
    ///             LightOp::new(0, 0, 255, Some(Duration::from_millis(100))),
    ///         ],
    ///     ).await.unwrap();
    /// }
    /// ```
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
                    .map(|d| (d.as_millis() / 10).max(1))
                    .unwrap_or(0);

                if d > 255 {
                    return Err(anyhow!("The duration must be less than 2560 milliseconds"));
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

    /// Turns on the light.
    ///
    /// The light color is set by RGB value, each of which must be in range 0 to 255.
    /// The duration must be less than 2560 milliseconds.
    ///
    /// ```no_run
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = toio::Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Turns on the green light.
    ///     cube.light_on(0, 255, 0, None).await.unwrap();
    /// }
    /// ```
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

    /// Turns off the light.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use tokio::time::delay_for;
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = toio::Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Turns on the light.
    ///     cube.light_on(255, 255, 255, None).await.unwrap();
    ///
    ///     delay_for(Duration::from_secs(3)).await;
    ///
    ///     // Turns off the light.
    ///     cube.light_off().await.unwrap();
    /// }
    /// ```
    pub async fn light_off(&mut self) -> Result<()> {
        self.dev
            .write_msg(Light::Off(LightOff::new()), true)
            .await?;
        Ok(())
    }

    /// Connects to the cube.
    ///
    /// This must be called first before operating on the cube.
    ///
    /// ```no_run
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///
    ///     // Connects to the cube.
    ///     cube.connect().await.unwrap();
    /// }
    /// ```
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

    /// Subscribes to events.
    ///
    /// ```no_run
    /// use futures::prelude::*;
    /// use toio::{Cube, Event};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     let mut events = cube.events().await.unwrap();
    ///     while let Some(event) = events.next().await {
    ///         match event {
    ///             Event::Collision(collided) => println!("collided: {}", collided),
    ///             Event::Battery(remain) => println!("battery: {}%", remain),
    ///             _ => {},
    ///         }
    ///     }
    /// }
    /// ```
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

    /// Writes a raw message to the device.
    ///
    /// This is the low-level API that allows to directly write
    /// the protocol data structures defined in [`proto`][] to the cube device.
    /// Some data triggers events which can be retrieved by [`Cube::raw_msgs`][] or [`Cube::events`][].
    ///
    /// ```no_run
    /// use toio::{Cube, proto::*};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Move forward.
    ///     cube.write_msg(
    ///         Message::Motor(Motor::Simple(MotorSimple::new(
    ///             MotorId::Left,
    ///             MotorDir::Forward,
    ///             30,
    ///             MotorId::Right,
    ///             MotorDir::Forward,
    ///             30,
    ///         ))),
    ///         false,
    ///     ).await.unwrap();
    /// }
    /// ```
    pub async fn write_msg(&mut self, msg: Message, with_resp: bool) -> Result<()> {
        self.dev.write_msg(msg, with_resp).await?;
        Ok(())
    }

    /// Sends a read request to the device.
    ///
    /// This is the low-level API to request to read values in the device.
    /// This usually triggers events which can be retrieved by [`Cube::raw_msgs`][] or [`Cube::events`][].
    ///
    /// ```no_run
    /// use futures::prelude::*;
    /// use toio::{Cube, proto::*};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Subscribe to raw messages.
    ///     let mut msgs = cube.raw_msgs().await.unwrap();
    ///
    ///     // Send a read request for motor state to the cube.
    ///     cube.read_msg(&UUID_MOTION).await.unwrap();
    ///
    ///     // Receive the motor state, which is sent as response to the read request.
    ///     while let Some(msg) = msgs.next().await {
    ///         match msg {
    ///             Message::Motion(Motion::Detect(d)) => {
    ///                 println!("{:?}", d);
    ///                 break;
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn read_msg(&mut self, uuid: &Uuid) -> Result<()> {
        self.dev.read(uuid).await?;
        Ok(())
    }

    /// Subscribe to raw messages.
    ///
    /// This is the low-level API to subscribe to raw protocol messages from the cube device.
    ///
    /// ```no_run
    /// use futures::prelude::*;
    /// use toio::{Cube, proto::*};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cube = Cube::search().nearest().await.unwrap();
    ///     cube.connect().await.unwrap();
    ///
    ///     // Subscribe to raw messages.
    ///     let mut msgs = cube.raw_msgs().await.unwrap();
    ///
    ///     // Receive raw messages.
    ///     while let Some(msg) = msgs.next().await {
    ///         match msg {
    ///             Message::Motion(Motion::Detect(d)) => {
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn raw_msgs(&mut self) -> Result<MessageStream> {
        Ok(self
            .dev
            .subscribe_msg()?
            .filter_map(|msg| async move { msg.ok() })
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
        Event::Position(p) => {
            status.position = Some(p);
        }
        Event::StdId(p) => {
            status.std_id = Some(p);
        }
        _ => {}
    }
}

fn convert(msg: Message) -> Option<Vec<Event>> {
    match msg {
        Message::Id(Id::Pos(pos)) => Some(vec![Event::Position(Some(pos.into()))]),
        Message::Id(Id::Std(std)) => Some(vec![Event::StdId(Some(std.into()))]),
        Message::Id(Id::PosMissed) => Some(vec![Event::Position(None)]),
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
