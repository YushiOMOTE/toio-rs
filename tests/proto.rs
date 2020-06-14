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
