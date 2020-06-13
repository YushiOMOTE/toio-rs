use crate::ble::{Notifications, PeripheralOps, SearchOps, Uuid as ClientUuid};

use anyhow::{anyhow, bail, Context, Error, Result};
use futures::prelude::*;
use log::*;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use tokio::{sync::broadcast, time::timeout};

use core_bluetooth::central::*;
use core_bluetooth::central::{
    characteristic::{Characteristic, WriteKind},
    peripheral::Peripheral,
    AdvertisementData,
};
use core_bluetooth::uuid::Uuid;
use core_bluetooth::*;

const SEARCH_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const WRITE_TIMEOUT: Duration = Duration::from_secs(1);
const CHANNEL_CAPACITY: usize = 100000;

pub struct Adaptor {
    central: CentralManager,
    peripheral: Peripheral,
    rssi: i32,
    connected: Arc<AtomicBool>,
    characteristics: HashMap<Uuid, Characteristic>,
    backend: Arc<Backend>,
}

impl Adaptor {
    fn new(
        central: CentralManager,
        peripheral: Peripheral,
        rssi: i32,
        backend: Arc<Backend>,
    ) -> Self {
        let connected = Arc::new(AtomicBool::new(false));
        let mut rx = backend.subscribe();

        let id = peripheral.id();
        let conn = connected.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(event) = rx.recv().await {
                    match event {
                        Event::Connected(peripheral, _) if peripheral.id() == id => {
                            conn.store(true, Ordering::SeqCst);
                        }
                        Event::Disconnected(peripheral) if peripheral.id() == id => {
                            conn.store(false, Ordering::SeqCst);
                        }
                        _ => {}
                    }
                }
            }
        });

        Self {
            central,
            peripheral,
            rssi,
            connected,
            characteristics: HashMap::new(),
            backend,
        }
    }
}

#[async_trait::async_trait]
impl PeripheralOps for Adaptor {
    fn rssi(&self) -> i32 {
        self.rssi
    }

    async fn connect(&mut self) -> Result<()> {
        let mut rx = self.backend.subscribe();

        self.central.connect(&self.peripheral);

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

    async fn write(&mut self, uuid: &ClientUuid, value: &[u8], with_resp: bool) -> Result<()> {
        if !self.connected.load(Ordering::Relaxed) {
            bail!("Peripheral {} is not connected", self.peripheral.id());
        }

        let mut rx = self.backend.subscribe();

        let uuid = Uuid::from_bytes(uuid.0);
        let c = self
            .characteristics
            .get(&uuid)
            .ok_or_else(|| anyhow!("No such characteristic {}", uuid))?;
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

    fn subscribe(&mut self) -> Result<Notifications> {
        let rx = self.backend.subscribe();
        let id = self.peripheral.id();

        Ok(rx
            .into_stream()
            .filter_map(move |event| async move {
                match event {
                    Ok(Event::Value(p, c, value)) if p.id() == id => {
                        Some((ClientUuid(c.id().bytes()), value))
                    }
                    _ => None,
                }
            })
            .boxed())
    }
}

pub struct Searcher {
    backend: Arc<Backend>,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            backend: Arc::new(Backend::new()),
        }
    }
}

#[async_trait::async_trait]
impl SearchOps for Searcher {
    type Adaptor = Adaptor;

    async fn search(&mut self, uuid: &ClientUuid) -> Result<Vec<Self::Adaptor>> {
        let uuid = Uuid::from_bytes(uuid.0);

        let mut rx = self.backend.subscribe();

        self.backend
            .central()
            .get_peripherals_with_services(&[uuid.clone()]);
        self.backend.central().scan();

        let mut found = vec![];
        let central = self.backend.central().clone();
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
                            found.push(Adaptor::new(
                                central.clone(),
                                peripheral,
                                rssi,
                                self.backend.clone(),
                            ));
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

#[derive(Clone, Debug)]
pub enum Event {
    Discovered(Peripheral, AdvertisementData, i32),
    Connected(Peripheral, Vec<Characteristic>),
    Disconnected(Peripheral),
    Value(Peripheral, Characteristic, Vec<u8>),
    WriteRes(Peripheral, Characteristic, bool),
}

struct ConnectionManager;

impl ConnectionManager {
    fn new() -> Self {
        Self
    }

    fn handle_event(&self, event: CentralEvent) -> Result<Option<Event>> {
        trace!("Event: {:?}", event);

        match event {
            CentralEvent::ManagerStateChanged { new_state } => match new_state {
                ManagerState::Unsupported => {
                    bail!("Bluetooth is not supported");
                }
                ManagerState::Unauthorized => {
                    bail!("Not authorized to use Bluetooth");
                }
                ManagerState::PoweredOff => {
                    warn!("Bluetooth is disabled");
                }
                ManagerState::PoweredOn => {
                    debug!("Bluetooth is enabled.");
                }
                _ => {}
            },
            CentralEvent::PeripheralDiscovered {
                peripheral,
                advertisement_data,
                rssi,
            } => {
                debug!(
                    "Discovered peripheral {} ({:?})",
                    peripheral.id(),
                    advertisement_data.local_name()
                );
                if advertisement_data.is_connectable() != Some(false) {
                    return Ok(Some(Event::Discovered(
                        peripheral,
                        advertisement_data,
                        rssi,
                    )));
                }
            }
            CentralEvent::GetPeripheralsWithServicesResult {
                peripherals: _,
                tag: _,
            } => {}
            CentralEvent::PeripheralConnected { peripheral } => {
                peripheral.discover_services();
            }
            CentralEvent::PeripheralDisconnected {
                peripheral,
                error: _,
            } => {
                return Ok(Some(Event::Disconnected(peripheral)));
            }
            CentralEvent::PeripheralConnectFailed { peripheral, error } => {
                warn!(
                    "Couldn't connect to peripheral {}: {}",
                    peripheral.id(),
                    error
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "<no error>".into())
                );
                return Ok(Some(Event::Disconnected(peripheral)));
            }
            CentralEvent::ServicesDiscovered {
                peripheral,
                services,
            } => {
                if let Ok(services) = services {
                    for service in services {
                        peripheral.discover_characteristics(&service);
                    }
                }
            }
            CentralEvent::SubscriptionChangeResult {
                peripheral,
                characteristic: _,
                result,
            } => {
                if result.is_err() {
                    warn!(
                        "Couldn't subscribe to characteristic of {}",
                        peripheral.id()
                    );
                } else {
                    debug!("Subscribe to characteristic of {}", peripheral.id());
                }
            }
            CentralEvent::CharacteristicsDiscovered {
                peripheral,
                service: _,
                characteristics,
            } => match characteristics {
                Ok(characteristics) => {
                    for c in characteristics.iter() {
                        if c.properties().can_read() {
                            debug!("Read characteristic {} of {}", c.id(), peripheral.id());
                            peripheral.read_characteristic(&c);
                        }
                        if c.properties().can_notify() {
                            debug!(
                                "Subscribing to characteristic {} of {}",
                                c.id(),
                                peripheral.id()
                            );
                            peripheral.subscribe(&c);
                        }
                    }

                    return Ok(Some(Event::Connected(peripheral, characteristics)));
                }
                Err(err) => error!(
                    "Couldn't discover characteristics of {}: {}",
                    peripheral.id(),
                    err
                ),
            },
            CentralEvent::CharacteristicValue {
                peripheral,
                characteristic,
                value,
            } => {
                if let Ok(value) = value {
                    debug!(
                        "Received value of characteristic {} of peripheral {}: value={:?}",
                        characteristic.id(),
                        peripheral.id(),
                        value
                    );
                    return Ok(Some(Event::Value(peripheral, characteristic, value)));
                }
            }
            CentralEvent::WriteCharacteristicResult {
                peripheral,
                characteristic,
                result,
            } => {
                return Ok(Some(Event::WriteRes(
                    peripheral,
                    characteristic,
                    result.is_ok(),
                )));
            }
            _ => {}
        }

        Ok(None)
    }
}

struct Backend {
    central: Option<CentralManager>,
    thread: Option<std::thread::JoinHandle<Result<()>>>,
    tx: broadcast::Sender<Event>,
}

impl Backend {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (central, central_rx) = CentralManager::new();
        let manager = ConnectionManager::new();

        // Run the thread to convert sync to async.
        let client_tx = tx.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(event) = central_rx.recv() {
                if let Some(event) = manager.handle_event(event)? {
                    let _ = client_tx.send(event);
                }
            }
            Ok(())
        });

        Self {
            central: Some(central),
            thread: Some(thread),
            tx,
        }
    }

    fn central(&self) -> &CentralManager {
        self.central.as_ref().unwrap()
    }

    fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        // Drop central to stop receiving events
        self.central.take();

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
