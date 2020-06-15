use std::time::Duration;
use toio::{Cube, Note, SoundOp, SoundPresetId};
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Search for the nearest cube.
    let mut cube = Cube::search().nearest().await.unwrap();

    // Connect.
    cube.connect().await.unwrap();

    delay_for(Duration::from_secs(1)).await;

    // Play the preset sound.
    cube.play_preset(SoundPresetId::Enter).await.unwrap();

    delay_for(Duration::from_secs(2)).await;

    // Play as programmed.
    cube.play(
        3,
        vec![
            SoundOp::new(Note::C5, Duration::from_millis(500)),
            SoundOp::new(Note::A6, Duration::from_millis(500)),
        ],
    )
    .await
    .unwrap();

    delay_for(Duration::from_secs(4)).await;

    // Play the sound.
    cube.play(1, vec![SoundOp::new(Note::C5, Duration::from_millis(2000))])
        .await
        .unwrap();

    delay_for(Duration::from_secs(1)).await;

    // Stop the sound.
    cube.stop_sound().await.unwrap();

    delay_for(Duration::from_secs(1)).await;
}
