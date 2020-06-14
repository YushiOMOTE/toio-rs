use anyhow::{anyhow, bail, Error, Result};
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::{TryFrom, TryInto};

use crate::{ble::Uuid, decode::decode, encode::encode, uuid};

/// The UUID of the toio cube service.
pub const UUID_SERVICE: Uuid = uuid!("10b20100 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the id reader characteristic.
pub const UUID_ID: Uuid = uuid!("10b20101 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the motor characteristic.
pub const UUID_MOTOR: Uuid = uuid!("10b20102 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the light characteristic.
pub const UUID_LIGHT: Uuid = uuid!("10b20103 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the sound device characteristic.
pub const UUID_SOUND: Uuid = uuid!("10b20104 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the motion sensor characteristic.
pub const UUID_MOTION: Uuid = uuid!("10b20106 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the button characteristic.
pub const UUID_BUTTON: Uuid = uuid!("10b20107 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the battery characteristic of the battery.
pub const UUID_BATTERY: Uuid = uuid!("10b20108 5b3b 4571 9508 cf3efcd7bbae");

/// The UUID of the configuration characteristic.
pub const UUID_CONFIG: Uuid = uuid!("10b201ff 5b3b 4571 9508 cf3efcd7bbae");

/// Sound note.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Note {
    /// C 0
    C0 = 0,
    /// C# 0
    CS0,
    /// D 0
    D0,
    /// D# 0
    DS0,
    /// E 0
    E0,
    /// F 0
    F0,
    /// F# 0
    FS0,
    /// G 0
    G0,
    /// G# 0
    GS0,
    /// A 0
    A0,
    /// A# 0
    AS0,
    /// B 0
    B0,
    /// C 1
    C1,
    /// C# 1
    CS1,
    /// D 1
    D1,
    /// D# 1
    DS1,
    /// E 1
    E1,
    /// F 1
    F1,
    /// F# 1
    FS1,
    /// G 1
    G1,
    /// G# 1
    GS1,
    /// A 1
    A1,
    /// A# 1
    AS1,
    /// B 1
    B1,
    /// C 2
    C2,
    /// C# 2
    CS2,
    /// D 2
    D2,
    /// D# 2
    DS2,
    /// E 2
    E2,
    /// F 2
    F2,
    /// F# 2
    FS2,
    /// G 2
    G2,
    /// G# 2
    GS2,
    /// A 2
    A2,
    /// A# 2
    AS2,
    /// B 2
    B2,
    /// C 3
    C3,
    /// C# 3
    CS3,
    /// D 3
    D3,
    /// D# 3
    DS3,
    /// E 3
    E3,
    /// F 3
    F3,
    /// F# 3
    FS3,
    /// G 3
    G3,
    /// G# 3
    GS3,
    /// A 3
    A3,
    /// A# 3
    AS3,
    /// B 3
    B3,
    /// C 4
    C4,
    /// C# 4
    CS4,
    /// D 4
    D4,
    /// D# 4
    DS4,
    /// E 4
    E4,
    /// F 4
    F4,
    /// F# 4
    FS4,
    /// G 4
    G4,
    /// G# 4
    GS4,
    /// A 4
    A4,
    /// A# 4
    AS4,
    /// B 4
    B4,
    /// C 5
    C5,
    /// C# 5
    CS5,
    /// D 5
    D5,
    /// D# 5
    DS5,
    /// E 5
    E5,
    /// F 5
    F5,
    /// F# 5
    FS5,
    /// G 5
    G5,
    /// G# 5
    GS5,
    /// A 5
    A5,
    /// A# 5
    AS5,
    /// B 5
    B5,
    /// C 6
    C6,
    /// C# 6
    CS6,
    /// D 6
    D6,
    /// D# 6
    DS6,
    /// E 6
    E6,
    /// F 6
    F6,
    /// F# 6
    FS6,
    /// G 6
    G6,
    /// G# 6
    GS6,
    /// A 6
    A6,
    /// A# 6
    AS6,
    /// B 6
    B6,
    /// C 7
    C7,
    /// C# 7
    CS7,
    /// D 7
    D7,
    /// D# 7
    DS7,
    /// E 7
    E7,
    /// F 7
    F7,
    /// F# 7
    FS7,
    /// G 7
    G7,
    /// G# 7
    GS7,
    /// A 7
    A7,
    /// A# 7
    AS7,
    /// B 7
    B7,
    /// C 8
    C8,
    /// C# 8
    CS8,
    /// D 8
    D8,
    /// D# 8
    DS8,
    /// E 8
    E8,
    /// F 8
    F8,
    /// F# 8
    FS8,
    /// G 8
    G8,
    /// G# 8
    GS8,
    /// A 8
    A8,
    /// A# 8
    AS8,
    /// B 8
    B8,
    /// C 9
    C9,
    /// C# 9
    CS9,
    /// D 9
    D9,
    /// D# 9
    DS9,
    /// E 9
    E9,
    /// F 9
    F9,
    /// F# 9
    FS9,
    /// G 9
    G9,
    /// G# 9
    GS9,
    /// A 9
    A9,
    /// A# 9
    AS9,
    /// B 9
    B9,
    /// C 10
    C10,
    /// C# 10
    CS10,
    /// D 10
    D10,
    /// D# 10
    DS10,
    /// E 10
    E10,
    /// F 10
    F10,
    /// F# 10
    FS10,
    /// G 10
    G10,
    /// No sound.
    NoSound,
}

macro_rules! msg {
    ($uuid:expr;
     $(#[$attr:meta])?pub enum $name:tt {
        $(
            $(#[$vattr:meta])?
            $variant:ident$(($value:ident))? = $id:literal,
        )*
    }) => {
        $(#[$attr])?
        #[derive(Debug, Clone, PartialEq, Eq, new)]
        pub enum $name {
            $(
                $(#[$vattr])?
                $variant$(($value))?,
            )*
        }

        #[allow(non_snake_case)]
        impl TryFrom<&[u8]> for $name {
            type Error = Error;

            fn try_from(v: &[u8]) -> Result<Self> {
                match v.get(0) {
                    $(Some($id) => Ok(Self::$variant$((decode::<$value>(&v[1..])?))? ),)*
                    Some(v) => Err(anyhow!("Invalid type {} for {}", v, stringify!(Self))),
                    None => Err(anyhow!("Empty bytes for {}", stringify!(Self))),
                }
            }
        }

        impl TryFrom<Vec<u8>> for $name {
            type Error = Error;

            fn try_from(v: Vec<u8>) -> Result<Self> {
                Self::try_from(&v as &[u8])
            }
        }

        #[allow(non_snake_case, unused_mut)]
        impl TryFrom<$name> for Vec<u8> {
            type Error = Error;

            fn try_from(v: $name) -> Result<Self> {
                match &v {
                    $($name::$variant$(($value))? => {
                        let mut buf = vec![$id];
                        $(encode(&mut buf, &$value)?;)?
                        Ok(buf)
                    },)*
                }
            }
        }

        impl TryFrom<$name> for (Uuid, Vec<u8>) {
            type Error = Error;

            fn try_from(v: $name) -> Result<Self> {
                Ok(($uuid, v.try_into()?))
            }
        }
    };
}

/// Position id
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct IdPos {
    /// The x coordinate of the cube.
    pub cube_x: u16,
    /// The y coordinate of the cube.
    pub cube_y: u16,
    /// The angle of the cube.
    pub cube_angle: u16,
    /// The x coordinate of the cube.
    pub sensor_x: u16,
    /// The y coordinate of the cube.
    pub sensor_y: u16,
    /// The angle of the cube.
    pub sensor_angle: u16,
}

/// Standard id
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct IdStd {
    /// The value of the standard id.
    pub value: u32,
    /// The angle of the cube.
    pub angle: u16,
}

msg!(
    UUID_ID;

    #[doc = "Message from the id reader."]
    pub enum Id {
        #[doc = "The content of the position id."]
        Pos(IdPos) = 0x01,
        #[doc = "The content of the standard id."]
        Std(IdStd) = 0x02,
        #[doc = "Indicates the cube goes out of the positionn id area."]
        PosMissed = 0x03,
        #[doc = "Indicates the cube goes out of the standard id area."]
        StdMissed = 0x04,
    }
);

/// Posture of the cube.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Posture {
    /// The top side of the cube is up.
    HeadUp = 0x01,
    /// The bottom side of the cube is up.
    BottomUp = 0x02,
    /// The back side of the cube is up.
    BackUp = 0x03,
    /// The front side of the cube is up.
    FrontUp = 0x04,
    /// The right side of the cube is up.
    RightSideUp = 0x05,
    /// The let side of the cube is up.
    LeftSideUp = 0x06,
}

/// The state of the motion sensor.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotionDetect {
    /// Set if the cube is level to the ground.
    pub level: bool,
    /// Set if the cube collides with an object.
    pub collision: bool,
    /// Set if the cube is tapped twice.
    pub double_tap: bool,
    /// The posture of the cube.
    pub posture: Posture,
}

msg!(
    UUID_MOTION;

    #[doc = "Message from the motion sensor."]
    pub enum Motion {
        #[doc = "The state of the motion sensor."]
        Detect(MotionDetect) = 0x01,
    }
);

/// The state of the button.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ButtonState {
    /// The button is not pressed.
    Released = 0x00,
    /// The button is pressed.
    Pressed = 0x80,
}

msg!(
    UUID_BUTTON;

    #[doc = "Message from the button."]
    pub enum Button {
        #[doc = "The state of the button."]
        Func(ButtonState) = 0x01,
    }
);

/// The id of the motor.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MotorId {
    /// The left motor.
    Left = 0x01,
    /// The right motor.
    Right = 0x02,
}

/// The direction of the motor rotation.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MotorDir {
    /// Rotate forward.
    Forward = 0x01,
    /// Rotate backward.
    Backward = 0x02,
}

/// The simple request to rotate the motor.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorSimple {
    /// The id of the first motor.
    pub motor1: MotorId,
    /// The rotation direction of the first motor.
    pub dir1: MotorDir,
    /// The rotation speed of the first motor.
    pub speed1: u8,
    /// The id of the second motor.
    pub motor2: MotorId,
    /// The rotation direction of the second motor.
    pub dir2: MotorDir,
    /// The rotation speed of the second motor.
    pub speed2: u8,
}

/// The request to rotate the motor with duration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorTimed {
    /// The id of the first motor.
    pub motor1: MotorId,
    /// The rotation direction of the first motor.
    pub dir1: MotorDir,
    /// The rotation speed of the first motor.
    pub speed1: u8,
    /// The id of the second motor.
    pub motor2: MotorId,
    /// The rotation direction of the second motor.
    pub dir2: MotorDir,
    /// The rotation speed of the second motor.
    pub speed2: u8,
    /// The duration to keep rotating the motor in milliseconds.
    pub duration: u8,
}

/// The type of the movement.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveType {
    /// The cube goes to the target position, spinning accordingly.
    Curve = 0x00,
    /// The cube goes to the target position without going backward.
    ForwardOnly = 0x01,
    /// The cube spins first and then goes straight to the target position.
    Straight = 0x02,
}

/// The change of the speed.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SpeedChange {
    /// The speed is constant.
    Const = 0x00,
    /// The speed increases.
    Acc = 0x01,
    /// The speed decreases.
    Dec = 0x02,
    /// The speed increases first then decreases.
    AccDec = 0x03,
}

/// The request to move to the specified position.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorTarget {
    /// The request id to find the corresponding response to this request.
    pub id: u8,
    /// The timeout in seconds.
    pub timeout: u8,
    /// The type of the movement.
    pub move_type: MoveType,
    /// The maximum speed.
    pub max_speed: u8,
    /// The change of the speed during movement.
    pub speed_change: SpeedChange,
    /// Unused.
    #[new(default)]
    pub reserved: u8,
    /// The x coordinate of the target position.
    pub x: u16,
    /// The y coordinate of the target position.
    pub y: u16,
    /// The angle of the cube at the target position.
    pub angle: u16,
}

/// The option for additional requests.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum WriteOpt {
    /// The new request overwrites the pending request.
    Overwrite = 0x00,
    /// The new request is scheduled after the pending request if condition met.
    Append = 0x01,
}

/// The target position.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct Target {
    /// The x coordinate of the target position.
    pub x: u16,
    /// The y coordinate of the target position.
    pub y: u16,
    /// The angle of the cube at the target position.
    pub angle: u16,
}

/// The request to visit multiple target positions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorMultiTarget {
    /// The request id to find the corresponding response to this request.
    pub id: u8,
    /// The timeout in seconds.
    pub timeout: u8,
    /// The type of the movement.
    pub move_type: MoveType,
    /// The maximum speed.
    pub max_speed: u8,
    /// The change of the speed during movement.
    pub speed_change: SpeedChange,
    /// Unused.
    #[new(default)]
    pub reserved: u8,
    /// The option for additional requests.
    pub writeopt: WriteOpt,
    /// The list of target positions to visit.
    pub targets: Vec<Target>,
}

/// The direction of the cube rotation/translation.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MotorCubeDir {
    /// Forward.
    Forward = 0x00,
    /// Backward.
    Backward = 0x01,
}

/// The priority of translation/rotation speed.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MotorPriority {
    /// The translation speed is prioritized.
    Translation = 0x00,
    /// The rotation speed is prioritized.
    Rotation = 0x01,
}

/// The request to move the cube with acceleration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorAcc {
    /// The request id to find the corresponding response to this request.
    pub id: u8,
    /// The speed of the cube.
    pub speed: u8,
    /// The acceleration of the cube.
    pub acc: u8,
    /// The rotation speed of the cube.
    pub rotate_speed: u16,
    /// The rotation direction of the cube.
    pub rotate_dir: MotorCubeDir,
    /// The translation direction of the cube.
    pub trans_dir: MotorCubeDir,
    /// The priority of translation/rotation speed.
    pub prio: MotorPriority,
    /// The duration to keep moving the cube in seconds.
    pub duration: u8,
}

/// The result value of the request.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TargetResValue {
    /// Succeeded.
    Ok = 0x00,
    /// The request timed out.
    Timeout = 0x01,
    /// The cube is out of the area of position id.
    IdMissed = 0x02,
    /// The given parameters were invalid.
    InvalidParam = 0x03,
    /// The state of the cube went invalid (e.g. powered off during movement).
    InvalidState = 0x04,
    /// The new request was written.
    OtherWrite = 0x05,
    /// The request was not supported.
    Unsupported = 0x06,
    /// The number of the pending requests exceeds the capacity.
    Full = 0x07,
}

/// The response to the request with target position.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorTargetRes {
    /// The request id.
    pub id: u8,
    /// The result value of the request.
    pub res: TargetResValue,
}

msg!(
    UUID_MOTOR;

    #[doc = "Message from/to the motor."]
    pub enum Motor {
        #[doc = "Simple request."]
        Simple(MotorSimple) = 0x01,
        #[doc = "Request with timeout."]
        Timed(MotorTimed) = 0x02,
        #[doc = "Request with target position."]
        Target(MotorTarget) = 0x03,
        #[doc = "Request with multiple target positions."]
        MultiTarget(MotorMultiTarget) = 0x04,
        #[doc = "Request with acceleration."]
        Acc(MotorAcc) = 0x05,
        #[doc = "Response to the request with target."]
        TargetRes(MotorTargetRes) = 0x83,
        #[doc = "Response to the request with multiple target."]
        MultiTargetRes(MotorTargetRes) = 0x84,
    }
);

/// Turns off the specified light.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct LightOff {
    /// The number of lights to turn off.
    #[new(value = "1")]
    pub num: u8,
    /// The id of the light to turn off.
    #[new(value = "1")]
    pub id: u8,
}

/// Turns on the specified light.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct LightOn {
    /// The duration for the light to be on.
    pub duration: u8,
    /// The number of the lights to turn on.
    #[new(value = "1")]
    pub num: u8,
    /// The id of the light to turn on.
    #[new(value = "1")]
    pub id: u8,
    /// The level of the red light.
    pub red: u8,
    /// The level of the green light.
    pub green: u8,
    /// The level of the blue light.
    pub blue: u8,
}

/// Control lights in accordance with the list of operations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct LightControl {
    /// The repeat count.
    pub repeat: u8,
    /// The number of operations.
    pub num: u8,
    /// The list of operations for lights.
    pub ops: Vec<LightOn>,
}

msg!(
    UUID_LIGHT;

    #[doc = "Message to lights."]
    pub enum Light {
        #[doc = "Turns off all the lights."]
        AllOff = 0x01,
        #[doc = "Turns off a light."]
        Off(LightOff) = 0x02,
        #[doc = "Turns on a light."]
        On(LightOn) = 0x03,
        #[doc = "Control lights with the list of operations."]
        Control(LightControl) = 0x04,
    }
);

/// The id of preset sound.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SoundPresetId {
    /// Enter.
    Enter = 0,
    /// Selected.
    Selected = 1,
    /// Cancel.
    Cancel = 2,
    /// Cursor.
    Cursor = 3,
    /// Mat in.
    MatIn = 4,
    /// Mat out.
    MatOut = 5,
    /// Get 1.
    Get1 = 6,
    /// Get 2.
    Get2 = 7,
    /// Get 3.
    Get3 = 8,
    /// Effect 1.
    Effect1 = 9,
    /// Effect 2.
    Effect2 = 10,
}

/// Plays preset sound.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct SoundPreset {
    /// The id of preset sound to play.
    pub id: SoundPresetId,
    /// The sound volume.
    pub vol: u8,
}

/// Plays sound with specified note.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct SoundOp {
    /// The duration to play the sound.
    pub duration: u8,
    /// The sound note.
    pub note: Note,
    /// The sound volume.
    pub vol: u8,
}

/// Plays sound in accordance with the list of operations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct SoundPlay {
    /// The repeat count.
    pub repeat: u8,
    /// The number of operations.
    pub num: u8,
    /// The list of operations to play sounds.
    pub ops: Vec<SoundOp>,
}

msg!(
    UUID_SOUND;

    #[doc = "Message to the sound device."]
    pub enum Sound {
        #[doc = "Stops."]
        Stop = 0x01,
        #[doc = "Plays the preset sound."]
        Preset(SoundPreset) = 0x02,
        #[doc = "Plays the sound with the list of operations.x"]
        Play(SoundPlay) = 0x03,
    }
);

/// Requests the protocol version.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct ConfigVersion {
    /// Unused.
    #[new(default)]
    pub reserve: u8,
}

/// Changes the settings of level detection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct ConfigLevel {
    /// Unused.
    #[new(default)]
    pub reserved: u8,
    /// The threashold for level detection.
    pub threshold: u8,
}

/// Changes the settings of collision detection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct ConfigCollision {
    /// Unused.
    #[new(default)]
    pub reserved: u8,
    /// The threashold for collision detection.
    pub threshold: u8,
}

/// Changes the settings of double-tap detection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct ConfigDoubleTap {
    /// Unused.
    #[new(default)]
    pub reserved: u8,
    /// The threashold for double-tap detection.
    pub interval: u8,
}

/// The protocol version information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct ConfigVersionRes {
    /// Unused.
    #[new(default)]
    pub reserved: u8,
    /// The protocol version.
    pub version: String,
}

msg!(
    UUID_CONFIG;

    #[doc = "Message from/to configuration."]
    pub enum Config {
        #[doc = "Requests protocol version."]
        Version(ConfigVersion) = 0x01,
        #[doc = "Changes the settings of level detection."]
        Level(ConfigLevel) = 0x02,
        #[doc = "Changes the settings of collision detection."]
        Collision(ConfigCollision) = 0x03,
        #[doc = "Changes the settings of double-tap detection."]
        DoubleTap(ConfigDoubleTap) = 0x04,
        #[doc = "The protocol version information."]
        VersionRes(ConfigVersionRes) = 0x81,
    }
);

/// Message read/written from/to characteristics.
#[derive(Debug, Clone, PartialEq, Eq, new)]
pub enum Message {
    /// Message for id reader.
    Id(Id),
    /// Message for motion sensor.
    Motion(Motion),
    /// Message for button.
    Button(Button),
    /// Message for battery.
    Battery(u8),
    /// Message for motor.
    Motor(Motor),
    /// Message for light.
    Light(Light),
    /// Message for sound device.
    Sound(Sound),
    /// Message for configuration.
    Config(Config),
}

fn unpack_battery(v: &[u8]) -> Result<u8> {
    v.get(0)
        .cloned()
        .ok_or_else(|| anyhow!("Battery field is empty"))
}

impl TryFrom<(Uuid, &[u8])> for Message {
    type Error = Error;

    fn try_from((uuid, buf): (Uuid, &[u8])) -> Result<Self> {
        let msg = match uuid {
            UUID_ID => Message::Id(buf.try_into()?),
            UUID_MOTION => Message::Motion(buf.try_into()?),
            UUID_BUTTON => Message::Button(buf.try_into()?),
            UUID_BATTERY => Message::Battery(unpack_battery(buf)?),
            UUID_MOTOR => Message::Motor(buf.try_into()?),
            UUID_LIGHT => Message::Light(buf.try_into()?),
            UUID_SOUND => Message::Sound(buf.try_into()?),
            UUID_CONFIG => Message::Config(buf.try_into()?),
            uuid => bail!("Unknown uuid: {}", uuid),
        };
        Ok(msg)
    }
}

impl TryFrom<(Uuid, Vec<u8>)> for Message {
    type Error = Error;

    fn try_from((uuid, buf): (Uuid, Vec<u8>)) -> Result<Self> {
        (uuid, &buf as &[u8]).try_into()
    }
}

impl TryFrom<Message> for (Uuid, Vec<u8>) {
    type Error = Error;

    fn try_from(msg: Message) -> Result<Self> {
        let v = match msg {
            Message::Id(v) => (UUID_ID, v.try_into()?),
            Message::Motion(v) => (UUID_MOTION, v.try_into()?),
            Message::Button(v) => (UUID_BUTTON, v.try_into()?),
            Message::Battery(v) => (UUID_BATTERY, vec![v]),
            Message::Motor(v) => (UUID_MOTOR, v.try_into()?),
            Message::Light(v) => (UUID_LIGHT, v.try_into()?),
            Message::Sound(v) => (UUID_SOUND, v.try_into()?),
            Message::Config(v) => (UUID_CONFIG, v.try_into()?),
        };
        Ok(v)
    }
}
