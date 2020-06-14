use std::convert::TryInto;
use toio::proto::*;

#[test]
fn test_id() {
    let p: Vec<u8> = Id::Pos(IdPos::new(1, 2, 3, 4, 5, 6)).try_into().unwrap();
    assert_eq!(
        p,
        vec![0x01, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00]
    );
    let p: Id = p.try_into().unwrap();
    assert_eq!(p, Id::Pos(IdPos::new(1, 2, 3, 4, 5, 6)));
}

#[test]
fn test_motion() {
    let p: Vec<u8> = Motion::Detect(MotionDetect::new(false, true, false, Posture::FrontUp))
        .try_into()
        .unwrap();
    assert_eq!(p, vec![0x01, 0x00, 0x01, 0x00, 0x04]);
    let p: Motion = p.try_into().unwrap();
    assert_eq!(
        p,
        Motion::Detect(MotionDetect::new(false, true, false, Posture::FrontUp))
    );
}

#[test]
fn test_button() {
    let p: Vec<u8> = Button::Func(ButtonState::Pressed).try_into().unwrap();
    assert_eq!(p, vec![0x01, 0x80]);
    let p: Button = p.try_into().unwrap();
    assert_eq!(p, Button::Func(ButtonState::Pressed));
}

#[test]
fn test_version() {
    let p: Vec<u8> = Config::VersionRes(ConfigVersionRes::new("testXY".into()))
        .try_into()
        .unwrap();
    assert_eq!(p, vec![0x81, 0x00, 0x74, 0x65, 0x73, 0x74, 0x58, 0x59]);
    let p: Config = p.try_into().unwrap();
    assert_eq!(
        p,
        Config::VersionRes(ConfigVersionRes::new("testXY".into()))
    );
}

#[test]
fn test_light() {
    let l = Light::Ctrl(LightCtrl::new(
        0,
        2,
        vec![LightOn::new(1, 2, 3, 4), LightOn::new(5, 6, 7, 8)],
    ));
    let p: Vec<u8> = l.clone().try_into().unwrap();
    assert_eq!(
        p,
        vec![
            0x04, 0x00, 0x02, 0x01, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x01, 0x01, 0x06, 0x07,
            0x08
        ]
    );
    let p: Light = p.try_into().unwrap();
    assert_eq!(p, l);
}
