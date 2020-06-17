use anyhow::{bail, Result};
use core_bluetooth::central::*;
use core_bluetooth::*;
use core_bluetooth::{
    central::{characteristic::Characteristic, peripheral::Peripheral, AdvertisementData},
    uuid::Uuid,
};
use futures::{
    future::{abortable, AbortHandle},
    prelude::*,
};
use log::*;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
        Arc,
    },
    time::Duration,
};
use tokio::sync::{broadcast, mpsc};

const CHANNEL_CAPACITY: usize = 100000;

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
    Discover(Uuid),
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
                InnerMsg::Discover(uuid) => {
                    self.central.get_peripherals_with_services(&[uuid]);
                    self.central.scan();
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
                    for c in characteristics
                        .iter()
                        .filter(|c| c.properties().can_notify())
                    {
                        debug!(
                            "Subscribing to characteristic {} of {}",
                            c.id(),
                            peripheral.id()
                        );
                        peripheral.subscribe(&c);
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

pub struct ConnectionManager {
    thread: Option<std::thread::JoinHandle<Result<()>>>,
    stop: Arc<AtomicBool>,
    client_tx: broadcast::Sender<Event>,
    manager_tx: mpsc::UnboundedSender<InnerMsg>,
    inner_handle: AbortHandle,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (manager_tx, manager_rx) = mpsc::unbounded_channel();
        let (client_tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (central, central_rx) = CentralManager::new();

        let mut inner = Inner::new(central, client_tx.clone(), manager_rx);
        let (inner, inner_handle) = abortable(async move {
            if let Err(e) = inner.run().await {
                error!("Error in connection manager: {}", e);
            }
        });
        tokio::spawn(inner);

        // Run the thread to convert sync to async.
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let tx = manager_tx.clone();
        let thread = std::thread::spawn(move || {
            loop {
                match central_rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(event) => {
                        let _ = tx.send(InnerMsg::Event(event));
                    }
                    Err(RecvTimeoutError::Disconnected) | Err(RecvTimeoutError::Timeout)
                        if stopped.load(Ordering::Relaxed) =>
                    {
                        break;
                    }
                    _ => {}
                }
            }
            Ok(())
        });

        Self {
            thread: Some(thread),
            stop,
            client_tx,
            manager_tx,
            inner_handle,
        }
    }

    pub fn discover(&self, uuid: &Uuid) {
        let _ = self.manager_tx.send(InnerMsg::Discover(uuid.clone()));
    }

    pub fn connect(&self, p: &Peripheral) {
        let _ = self.manager_tx.send(InnerMsg::Connect(p.clone()));
    }

    pub fn disconnect(&self, p: &Peripheral) {
        let _ = self.manager_tx.send(InnerMsg::Disconnect(p.clone()));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.client_tx.subscribe()
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        // Abort inner thread.
        self.inner_handle.abort();

        // Exit the thread to poll events.
        self.stop.store(true, Ordering::Relaxed);

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
