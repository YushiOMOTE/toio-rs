use toio::{
    ble::{self, PeripheralOps, PeripheralOpsExt},
    proto::*,
};

use std::time::Duration;
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut searcher = ble::searcher();
    let mut peripherals = searcher.search(&UUID_SERVICE).await.unwrap();
    let mut peripheral = peripherals.pop().unwrap();

    peripheral.connect().await.unwrap();

    delay_for(Duration::from_secs(2)).await;

    peripheral
        .write_msg(
            Motor::Simple(MotorSimple::new(
                MotorId::Left,
                MotorDir::Forward,
                30,
                MotorId::Right,
                MotorDir::Forward,
                30,
            )),
            false,
        )
        .await
        .unwrap();

    delay_for(Duration::from_secs(2)).await;
}
