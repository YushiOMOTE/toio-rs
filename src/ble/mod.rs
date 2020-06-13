use anyhow::{Context, Error, Result};
use std::convert::TryInto;
use tokio::sync::broadcast;

#[macro_export]
macro_rules! uuid {
    ($hex:literal) => {
        $crate::ble::Uuid(hex_literal::hex!($hex))
    };
}

/// Uuid for services or characteristics.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Uuid(pub [u8; 16]);

/// Callback to receive values from peripherals.
pub type Notifications = broadcast::Receiver<(Uuid, Vec<u8>)>;

/// The interface for platform-specific BLE searcher.
#[async_trait::async_trait]
pub trait SearchOps {
    type Adaptor;

    /// Search for peripherals.
    async fn search(&mut self, uuid: &Uuid) -> Result<Vec<Self::Adaptor>>;
}

/// The interface for platform-specific BLE peripheral.
#[async_trait::async_trait]
pub trait PeripheralOps {
    // Rssi
    fn rssi(&self) -> i32;

    /// Connect to the peripheral.
    async fn connect(&mut self) -> Result<()>;

    /// Write with/without response.
    async fn write(&mut self, uuid: &Uuid, value: &[u8], with_resp: bool) -> Result<()>;

    /// Write protocol message.
    async fn write_msg<T>(&mut self, uuid: &Uuid, value: T, with_resp: bool) -> Result<()>
    where
        T: TryInto<Vec<u8>, Error = Error> + Send,
    {
        let value: Vec<u8> = value.try_into().context("Couldn't pack message")?;
        self.write(uuid, &value, with_resp).await?;
        Ok(())
    }

    /// Subscribe to the peripheral.
    fn subscribe(&mut self) -> Result<Notifications>;
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{Adaptor, Searcher};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{Adaptor, Searcher};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{Adaptor, Searcher};
