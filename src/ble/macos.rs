use crate::ble::{self, PeripheralOps, SearchOps, ValueStream};

use anyhow::{anyhow, bail, Context, Error, Result};
use futures::{
    future::{abortable, AbortHandle},
    prelude::*,
};
use log::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use tokio::{
    sync::{broadcast, mpsc},
    time::timeout,
};

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

    async fn read(&mut self, uuid: &ble::Uuid) -> Result<()> {
        let uuid = Uuid::from_bytes(uuid.0);
        let c = self.ch(&uuid)?;
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

        self.manager
            .central()
            .get_peripherals_with_services(&[uuid.clone()]);
        self.manager.central().scan();

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

#[derive(Clone, Debug)]
pub enum Event {
    Discovered(Peripheral, AdvertisementData, i32),
    Connected(Peripheral, Vec<Characteristic>),
    Disconnected(Peripheral),
    Value(Peripheral, Characteristic, Vec<u8>),
    WriteRes(Peripheral, Characteristic, bool),
}

enum InnerMsg {
    Connect(Peripheral),
    Disconnect(Peripheral),
    Event(CentralEvent),
}

struct Inner {
    central: CentralManager,
    client_tx: broadcast::Sender<Event>,
    manager_rx: mpsc::UnboundedReceiver<InnerMsg>,
    connected: HashSet<Peripheral>,
}

impl Inner {
    fn new(
        central: CentralManager,
        client_tx: broadcast::Sender<Event>,
        manager_rx: mpsc::UnboundedReceiver<InnerMsg>,
    ) -> Self {
        Self {
            central,
            client_tx,
            manager_rx,
            connected: HashSet::new(),
        }
    }

    async fn run(&mut self) -> Result<()> {
        while let Some(msg) = self.manager_rx.next().await {
            match msg {
                InnerMsg::Connect(p) => {
                    self.connected.insert(p.clone());
                    self.central.connect(&p);
                }
                InnerMsg::Disconnect(p) => {
                    self.connected.remove(&p);
                    self.central.cancel_connect(&p);
                }
                InnerMsg::Event(e) => {
                    self.on_event(e).await?;
                }
            }
        }

        Ok(())
    }

    async fn on_event(&mut self, event: CentralEvent) -> Result<()> {
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
                    let _ = self.client_tx.send(Event::Discovered(
                        peripheral,
                        advertisement_data,
                        rssi,
                    ));
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
                if self.connected.contains(&peripheral) {
                    warn!("Reconnecting to {}", peripheral.id());
                    self.central.connect(&peripheral);
                } else {
                    let _ = self.client_tx.send(Event::Disconnected(peripheral));
                }
            }
            CentralEvent::PeripheralConnectFailed { peripheral, error } => {
                if self.connected.contains(&peripheral) {
                    warn!(
                        "Couldn't connect to peripheral {}: {}",
                        peripheral.id(),
                        error
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "<no error>".into())
                    );
                    self.central.connect(&peripheral);
                }
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

                    let _ = self
                        .client_tx
                        .send(Event::Connected(peripheral, characteristics));
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
                    let _ = self
                        .client_tx
                        .send(Event::Value(peripheral, characteristic, value));
                }
            }
            CentralEvent::WriteCharacteristicResult {
                peripheral,
                characteristic,
                result,
            } => {
                let _ = self.client_tx.send(Event::WriteRes(
                    peripheral,
                    characteristic,
                    result.is_ok(),
                ));
            }
            _ => {}
        }

        Ok(())
    }
}

struct ConnectionManager {
    central: Option<CentralManager>,
    thread: Option<std::thread::JoinHandle<Result<()>>>,
    client_tx: broadcast::Sender<Event>,
    manager_tx: mpsc::UnboundedSender<InnerMsg>,
    inner_handle: AbortHandle,
}

impl ConnectionManager {
    fn new() -> Self {
        let (manager_tx, manager_rx) = mpsc::unbounded_channel();
        let (client_tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (central, central_rx) = CentralManager::new();

        let mut inner = Inner::new(central.clone(), client_tx.clone(), manager_rx);
        let (inner, inner_handle) = abortable(async move {
            if let Err(e) = inner.run().await {
                error!("Error in connection manager: {}", e);
            }
        });
        tokio::spawn(inner);

        // Run the thread to convert sync to async.
        let tx = manager_tx.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(event) = central_rx.recv() {
                let _ = tx.send(InnerMsg::Event(event));
            }
            Ok(())
        });

        Self {
            central: Some(central),
            thread: Some(thread),
            client_tx,
            manager_tx,
            inner_handle,
        }
    }

    fn central(&self) -> &CentralManager {
        self.central.as_ref().unwrap()
    }

    fn connect(&self, p: &Peripheral) {
        let _ = self.manager_tx.send(InnerMsg::Connect(p.clone()));
    }

    fn disconnect(&self, p: &Peripheral) {
        let _ = self.manager_tx.send(InnerMsg::Disconnect(p.clone()));
    }

    fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.client_tx.subscribe()
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        // Abort inner thread.
        self.inner_handle.abort();

        // Drop central to stop receiving events.
        self.central.take();

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
