use std::convert::TryInto;
use toio::proto::*;

#[test]
fn test_id() {
    let p: Vec<u8> = Id::Pos(IdPosition::new(1, 2, 3, 4, 5, 6))
        .try_into()
        .unwrap();
    assert_eq!(
        p,
        vec![0x01, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00]
    );
    let p: Id = p.try_into().unwrap();
    assert_eq!(p, Id::Pos(IdPosition::new(1, 2, 3, 4, 5, 6)));
}

#[test]
fn test_motion() {
    let p: Vec<u8> = Motion::Detect(MotionDetect::new(false, true, false, Ori::Front))
        .try_into()
        .unwrap();
    assert_eq!(p, vec![0x01, 0x00, 0x01, 0x00, 0x04]);
    let p: Motion = p.try_into().unwrap();
    assert_eq!(
        p,
        Motion::Detect(MotionDetect::new(false, true, false, Ori::Front))
    );
}

#[test]
fn test_button() {
    let p: Vec<u8> = Button::Func(ButtonState::Pressed).try_into().unwrap();
    assert_eq!(p, vec![0x01, 0x80]);
    let p: Button = p.try_into().unwrap();
    assert_eq!(p, Button::Func(ButtonState::Pressed));
}
