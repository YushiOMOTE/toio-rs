use anyhow::{Context, Error, Result};
use derive_new::new;
use futures::{prelude::*, stream::BoxStream};
use std::{
    convert::{TryFrom, TryInto},
    fmt::{self, Display},
    time::Duration,
};

/// Helper to construct [`ble::Uuid`][] from a string at compile time.
#[macro_export]
macro_rules! uuid {
    ($hex:literal) => {
        $crate::ble::Uuid(hex_literal::hex!($hex))
    };
}

/// Uuid for services or characteristics.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, new)]
pub struct Uuid(pub [u8; 16]);

impl Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = self.0;
        write!(f, "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15])
    }
}

/// Callback to receive values from peripherals.
pub type ValueStream = BoxStream<'static, (Uuid, Vec<u8>)>;

/// Callback to receive values from peripherals.
pub type MessageStream<T> = BoxStream<'static, Result<T>>;

/// Peripheral
pub type Peripheral = Box<dyn PeripheralOps + Send>;

/// Searcher
pub type Searcher = Box<dyn SearchOps + Send>;

/// The interface for platform-specific BLE searcher.
#[async_trait::async_trait]
pub trait SearchOps {
    /// Search for peripherals.
    async fn search(&mut self, uuid: &Uuid, timeout: Duration) -> Result<Vec<Peripheral>>;
}

/// The interface for platform-specific BLE peripheral.
#[async_trait::async_trait]
pub trait PeripheralOps {
    // Peripheral id.
    fn id(&self) -> &str;

    // Rssi
    fn rssi(&self) -> i32;

    /// Connect to the peripheral.
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect the peripheral.
    async fn disconnect(&mut self) -> Result<()>;

    /// Send a read request.
    async fn read(&mut self, uuid: &Uuid) -> Result<()>;

    /// Write with/without response.
    async fn write(&mut self, uuid: &Uuid, value: &[u8], with_resp: bool) -> Result<()>;

    /// Subscribe to the peripheral.
    fn subscribe(&mut self) -> Result<ValueStream>;
}

/// The interface to help reading/writing protocol messages.
#[async_trait::async_trait]
pub trait PeripheralOpsExt: PeripheralOps {
    /// Write protocol message.
    async fn write_msg<T>(&mut self, value: T, with_resp: bool) -> Result<()>
    where
        T: TryInto<(Uuid, Vec<u8>), Error = Error> + Send,
    {
        let (uuid, value): (Uuid, Vec<u8>) =
            value.try_into().context(format!("Couldn't pack message"))?;
        self.write(&uuid, &value, with_resp).await?;
        Ok(())
    }

    /// Subscribe to the peripheral parsing bytes to protocol messge.
    fn subscribe_msg<T>(&mut self) -> Result<MessageStream<T>>
    where
        T: TryFrom<(Uuid, Vec<u8>), Error = Error> + Send,
    {
        Ok(self
            .subscribe()?
            .map(|(uuid, value)| {
                (uuid.clone(), value).try_into().context(format!(
                    "Couldn't unpack message from characteristic {}",
                    uuid
                ))
            })
            .boxed())
    }
}

#[async_trait::async_trait]
impl<T> PeripheralOps for Box<T>
where
    T: PeripheralOps + ?Sized + Send,
{
    fn id(&self) -> &str {
        (**self).id()
    }

    fn rssi(&self) -> i32 {
        (**self).rssi()
    }

    async fn connect(&mut self) -> Result<()> {
        (**self).connect().await
    }

    async fn disconnect(&mut self) -> Result<()> {
        (**self).disconnect().await
    }

    async fn read(&mut self, uuid: &Uuid) -> Result<()> {
        (**self).read(uuid).await
    }

    async fn write(&mut self, uuid: &Uuid, value: &[u8], with_resp: bool) -> Result<()> {
        (**self).write(uuid, value, with_resp).await
    }

    fn subscribe(&mut self) -> Result<ValueStream> {
        (**self).subscribe()
    }
}

impl<T> PeripheralOpsExt for T where T: PeripheralOps {}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Create a platform-specific searcher instance.
pub fn searcher() -> Searcher {
    #[cfg(target_os = "linux")]
    use linux::searcher as s;
    #[cfg(target_os = "macos")]
    use macos::searcher as s;
    #[cfg(target_os = "windows")]
    use windows::searcher as s;

    s()
}
