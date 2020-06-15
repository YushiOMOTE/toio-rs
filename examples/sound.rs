use std::time::Duration;
use toio::{Cube, Note, Sound, SoundOp, SoundOps, SoundPresetId};
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for the nearest cube
    let mut cube = Cube::search().nearest().await.unwrap();

    // Connect
    cube.connect().await.unwrap();

    delay_for(Duration::from_secs(1)).await;

    cube.play(Sound::Preset(SoundPresetId::Enter))
        .await
        .unwrap();

    delay_for(Duration::from_secs(2)).await;

    cube.play(Sound::Ops(SoundOps::new(
        vec![
            SoundOp::new(Note::C5, Duration::from_millis(500)),
            SoundOp::new(Note::A6, Duration::from_millis(500)),
        ],
        3,
    )))
    .await
    .unwrap();

    delay_for(Duration::from_secs(4)).await;

    cube.play(Sound::Ops(SoundOps::new(
        vec![SoundOp::new(Note::C5, Duration::from_millis(2000))],
        1,
    )))
    .await
    .unwrap();

    delay_for(Duration::from_secs(1)).await;

    cube.stop_sound().await.unwrap();

    delay_for(Duration::from_secs(1)).await;
}
