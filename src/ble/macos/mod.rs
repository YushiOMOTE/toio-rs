use crate::ble::{self, PeripheralOps, SearchOps, ValueStream};

use anyhow::{anyhow, bail, Context, Error, Result};
use futures::prelude::*;
use log::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::timeout;

use core_bluetooth::{
    central::{
        characteristic::{Characteristic, WriteKind},
        peripheral::Peripheral,
    },
    uuid::Uuid,
};

mod connection;

use self::connection::{ConnectionManager, Event};

const SEARCH_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const WRITE_TIMEOUT: Duration = Duration::from_secs(1);

pub struct Adaptor {
    peripheral: Peripheral,
    rssi: i32,
    characteristics: HashMap<Uuid, Characteristic>,
    manager: Arc<ConnectionManager>,
}

impl Adaptor {
    fn new(peripheral: Peripheral, rssi: i32, manager: Arc<ConnectionManager>) -> Self {
        Self {
            peripheral,
            rssi,
            characteristics: HashMap::new(),
            manager,
        }
    }

    fn ch(&self, uuid: &Uuid) -> Result<&Characteristic> {
        let ch = self
            .characteristics
            .get(uuid)
            .ok_or_else(|| anyhow!("No such characteristic {}", uuid))?;
        Ok(ch)
    }
}

impl Drop for Adaptor {
    fn drop(&mut self) {
        self.manager.disconnect(&self.peripheral);
    }
}

#[async_trait::async_trait]
impl PeripheralOps for Adaptor {
    fn rssi(&self) -> i32 {
        self.rssi
    }

    async fn connect(&mut self) -> Result<()> {
        let mut rx = self.manager.subscribe();

        self.manager.connect(&self.peripheral);

        let id = self.peripheral.id();
        let connect = async {
            loop {
                let event = rx
                    .recv()
                    .await
                    .context("Internal channel closed while waiting for connection status")?;

                match event {
                    Event::Connected(peripheral, characteristics) => {
                        if peripheral.id() == id {
                            debug!("Connected to peripheral {}", peripheral.id());
                            self.characteristics = characteristics
                                .into_iter()
                                .map(|c| (c.id().clone(), c))
                                .collect();
                            return Ok::<_, Error>(());
                        }
                    }
                    _ => {}
                }
            }
        };

        timeout(CONNECT_TIMEOUT, connect).await??;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        let mut rx = self.manager.subscribe();

        self.manager.disconnect(&self.peripheral);

        let id = self.peripheral.id();
        let connect = async {
            loop {
                let event = rx
                    .recv()
                    .await
                    .context("Internal channel closed while waiting for disconnection result")?;

                match event {
                    Event::Disconnected(peripheral) => {
                        if peripheral.id() == id {
                            debug!("Disconnected peripheral {}", peripheral.id());
                            return Ok::<_, Error>(());
                        }
                    }
                    _ => {}
                }
            }
        };

        timeout(CONNECT_TIMEOUT, connect).await??;

        Ok(())
    }

    async fn read(&mut self, uuid: &ble::Uuid) -> Result<()> {
        let uuid = Uuid::from_bytes(uuid.0);
        let c = self.ch(&uuid)?;
        debug!("Sending read request to characteristic {}", c.id());
        self.peripheral.read_characteristic(c);
        Ok(())
    }

    async fn write(&mut self, uuid: &ble::Uuid, value: &[u8], with_resp: bool) -> Result<()> {
        let mut rx = self.manager.subscribe();

        let uuid = Uuid::from_bytes(uuid.0);
        let c = self.ch(&uuid)?;
        let w = if with_resp {
            WriteKind::WithResponse
        } else {
            WriteKind::WithoutResponse
        };
        debug!("Writing value to characteristic {}: {:?}", c.id(), value);
        self.peripheral.write_characteristic(c, value, w);

        if with_resp {
            let pid = self.peripheral.id();
            let cid = uuid;
            let mut ok = None;
            let resp = async {
                loop {
                    let event = rx
                        .recv()
                        .await
                        .context("Internal channel closed while waiting for write response")?;

                    match event {
                        Event::WriteRes(peripheral, characteristics, res_ok)
                            if peripheral.id() == pid && characteristics.id() == cid =>
                        {
                            ok = Some(res_ok);
                            break;
                        }
                        _ => {}
                    }
                }
                Ok::<_, Error>(())
            };

            timeout(WRITE_TIMEOUT, resp).await??;

            return match ok {
                Some(true) => Ok(()),
                Some(false) => bail!("Write error"),
                None => bail!("No response"),
            };
        }

        Ok(())
    }

    fn subscribe(&mut self) -> Result<ValueStream> {
        let rx = self.manager.subscribe();
        let id = self.peripheral.id();

        Ok(rx
            .into_stream()
            .filter_map(move |event| async move {
                match event {
                    Ok(Event::Value(p, c, value)) if p.id() == id => {
                        Some((ble::Uuid(c.id().bytes()), value))
                    }
                    _ => None,
                }
            })
            .boxed())
    }
}

pub fn searcher() -> ble::Searcher {
    Box::new(Searcher::new())
}

pub struct Searcher {
    manager: Arc<ConnectionManager>,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(ConnectionManager::new()),
        }
    }
}

#[async_trait::async_trait]
impl SearchOps for Searcher {
    async fn search(&mut self, uuid: &ble::Uuid) -> Result<Vec<ble::Peripheral>> {
        let uuid = Uuid::from_bytes(uuid.0);

        let mut rx = self.manager.subscribe();
        self.manager.discover(&uuid);

        let mut found = Vec::new();
        let discover = async {
            loop {
                let event = rx
                    .recv()
                    .await
                    .context("Internal channel closed while searching for device")?;

                match event {
                    Event::Discovered(peripheral, ad, rssi) => {
                        if ad.service_uuids().contains(&uuid) {
                            debug!("Discovered peripheral: {:?}", peripheral);
                            found.push(Box::new(Adaptor::new(
                                peripheral,
                                rssi,
                                self.manager.clone(),
                            )) as ble::Peripheral);
                        }
                    }
                    _ => {}
                }
            }

            #[allow(unreachable_code)]
            Ok::<_, Error>(())
        };

        if let Ok(e) = timeout(SEARCH_TIMEOUT, discover).await {
            e?
        }

        Ok(found)
    }
}
