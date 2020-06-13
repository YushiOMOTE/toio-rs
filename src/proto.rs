use anyhow::{anyhow, bail, Error, Result};
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::{TryFrom, TryInto};

use crate::{ble::Uuid, uuid};

pub const UUID_SERVICE: Uuid = uuid!("10b20100 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_ID: Uuid = uuid!("10b20101 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_MOTOR: Uuid = uuid!("10b20102 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_LIGHT: Uuid = uuid!("10b20103 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_SOUND: Uuid = uuid!("10b20104 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_MOTION: Uuid = uuid!("10b20106 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_BUTTON: Uuid = uuid!("10b20107 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_BATTERY: Uuid = uuid!("10b20108 5b3b 4571 9508 cf3efcd7bbae");
pub const UUID_CONFIG: Uuid = uuid!("10b201ff 5b3b 4571 9508 cf3efcd7bbae");

macro_rules! msg {
    (pub enum $name:tt {
        $($variant:tt$(($value:ident))? = $id:literal,)*
    }) => {
        #[derive(Debug, Clone, PartialEq, Eq, new)]
        pub enum $name {
            $($variant$(($value))?,)*
        }

        #[allow(non_snake_case)]
        impl TryFrom<&[u8]> for $name {
            type Error = Error;

            fn try_from(v: &[u8]) -> Result<Self> {
                match v.get(0) {
                    $(Some($id) => Ok(Self::$variant$((bincode::deserialize::<$value>(&v[1..])?))? ),)*
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
                        $(bincode::serialize_into(&mut buf, &$value)?;)?
                        Ok(buf)
                    },)*
                }
            }
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct IdPosition {
    pub cube_x: u16,
    pub cube_y: u16,
    pub cube_angle: u16,
    pub sensor_x: u16,
    pub sensor_y: u16,
    pub sensor_angle: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct IdStandard {
    pub value: u8,
    pub angle: u8,
}

msg!(
    pub enum Id {
        Pos(IdPosition) = 0x01,
        Std(IdStandard) = 0x02,
        PositionMissed = 0x03,
        StandardMissed = 0x04,
    }
);

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Ori {
    Top = 0x01,
    Bottom = 0x02,
    Back = 0x03,
    Front = 0x04,
    Right = 0x05,
    Left = 0x06,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotionDetect {
    pub even: bool,
    pub collision: bool,
    pub tap: bool,
    pub position: Ori,
}

msg!(
    pub enum Motion {
        Detect(MotionDetect) = 0x01,
    }
);

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ButtonState {
    Released = 0x00,
    Pressed = 0x80,
}

msg!(
    pub enum Button {
        Func(ButtonState) = 0x01,
    }
);

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MotorDir {
    Forward = 0x01,
    Back = 0x02,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorSimple {
    pub motor1: u8,
    pub dir1: MotorDir,
    pub speed1: u8,
    pub motor2: u8,
    pub dir2: MotorDir,
    pub speed2: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorTimed {
    pub motor1: u8,
    pub dir1: MotorDir,
    pub speed1: u8,
    pub motor2: u8,
    pub dir2: MotorDir,
    pub speed2: u8,
    pub duration: u8,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MovePattern {
    Spin1 = 0x00,
    Spin2 = 0x01,
    Spin3 = 0x02,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SpeedPattern {
    Const = 0x00,
    Acc = 0x01,
    Dec = 0x02,
    AccDec = 0x03,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorTarget {
    pub id: u8,
    pub timeout: u8,
    pub move_pattern: MovePattern,
    pub max_speed: u8,
    pub speed_pattern: SpeedPattern,
    pub reserved: u8,
    pub x: u16,
    pub y: u16,
    pub angle: u16,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum WriteOpt {
    Overwrite = 0x00,
    Append = 0x01,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct Target {
    pub x: u16,
    pub y: u16,
    pub angle: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorMultiTarget {
    pub id: u8,
    pub timeout: u8,
    pub move_pattern: MovePattern,
    pub max_speed: u8,
    pub speed_pattern: SpeedPattern,
    pub reserved: u8,
    pub writeopt: WriteOpt,
    pub targets: Vec<Target>,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MotorCubeDir {
    Forward = 0x00,
    Back = 0x01,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MotorPriority {
    Move = 0x00,
    Spin = 0x01,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorAcc {
    pub id: u8,
    pub speed: u8,
    pub acc: u8,
    pub spin_speed: u16,
    pub spin_dir: MotorCubeDir,
    pub move_dir: MotorCubeDir,
    pub prio: MotorPriority,
    pub duration: u8,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ResStatusValue {
    Ok = 0x00,
    Timeout = 0x01,
    IdMissed = 0x02,
    InvalidParam = 0x03,
    InvalidState = 0x04,
    OtherWrite = 0x05,
    Unsupported = 0x06,
    Full = 0x07,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct MotorResStatus {
    pub id: u8,
    pub res: ResStatusValue,
}

msg!(
    pub enum Motor {
        Simple(MotorSimple) = 0x01,
        Timed(MotorTimed) = 0x02,
        Target(MotorTarget) = 0x03,
        MultiTarget(MotorMultiTarget) = 0x04,
        Acc(MotorAcc) = 0x05,
        TargetRes(MotorResStatus) = 0x83,
        MultiTargetRes(MotorResStatus) = 0x84,
    }
);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct LightOff {
    pub num: u8,
    pub id: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct LightOn {
    pub duration: u8,
    pub num: u8,
    pub id: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct LightMultiOn {
    pub repeat: u8,
    pub num: u8,
    pub ops: Vec<LightOn>,
}

msg!(
    pub enum Light {
        AllOff = 0x01,
        Off(LightOff) = 0x02,
        On(LightOn) = 0x03,
        MultiOn(LightMultiOn) = 0x04,
    }
);

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SoundPresetId {
    Enter = 0,
    Selected = 1,
    Cancel = 2,
    Cursor = 3,
    MatIn = 4,
    MatOut = 5,
    Get1 = 6,
    Get2 = 7,
    Get3 = 8,
    Effect1 = 9,
    Effect2 = 10,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct SoundPreset {
    pub id: SoundPresetId,
    pub vol: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct SoundOp {
    pub duration: u8,
    pub note: u8,
    pub vol: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct SoundPlay {
    pub repeat: u8,
    pub num: u8,
    pub ops: Vec<SoundOp>,
}

msg!(
    pub enum Sound {
        Stop = 0x01,
        Preset(SoundPreset) = 0x02,
        Play(SoundPlay) = 0x03,
    }
);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct ProtocolVersion {
    #[new(default)]
    pub reserve: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct HorizontalDetection {
    #[new(default)]
    pub reserved: u8,
    pub threshold: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct CollisionDetection {
    #[new(default)]
    pub reserved: u8,
    pub threshold: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct DoubleTapDetection {
    #[new(default)]
    pub reserved: u8,
    pub interval: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, new)]
pub struct ProtocolVersionRes {
    pub reserved: u8,
    pub version: [u8; 5],
}

msg!(
    pub enum Config {
        ProtocolVersion(ProtocolVersion) = 0x01,
        HorizontalDetection(HorizontalDetection) = 0x02,
        CollisionDetection(CollisionDetection) = 0x03,
        DoubleTapDetection(DoubleTapDetection) = 0x04,
        ProtocolVersionRes(ProtocolVersionRes) = 0x81,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, new)]
pub enum Message {
    Id(Id),
    Motion(Motion),
    Button(Button),
    Battery(u8),
    Motor(Motor),
    Light(Light),
    Sound(Sound),
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
            uuid => bail!("Unknown uuid: {:?}", uuid),
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
