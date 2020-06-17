use crate::{
    ble::{self, PeripheralOps},
    proto, Cube,
};
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

/// Searcher to search cubes.
pub struct Searcher {
    searcher: ble::Searcher,
    timeout: Duration,
}

impl Searcher {
    /// Creates a new searcher instance.
    ///
    /// The default search timeout is 3 seconds.
    /// Use [`Searcher::new_with_timeout`][] to specify custom timeout.
    pub fn new() -> Self {
        Self::new_with_timeout(Duration::from_secs(3))
    }

    /// Creates a new searcher instance with timeout.
    pub fn new_with_timeout(timeout: Duration) -> Self {
        Self {
            searcher: ble::searcher(),
            timeout,
        }
    }

    /// Search for all cubes.
    pub async fn all(&mut self) -> Result<Vec<Cube>> {
        Ok(self
            .do_search()
            .await?
            .into_iter()
            .map(|a| Cube::new(a))
            .collect())
    }

    /// Finds the nearest cube.
    pub async fn nearest(&mut self) -> Result<Cube> {
        self.do_search()
            .await?
            .into_iter()
            .map(|c| c)
            .max_by(|a, b| a.rssi().cmp(&b.rssi()))
            .map(|a| Cube::new(a))
            .ok_or_else(|| anyhow!("No cube found"))
    }

    async fn do_search(&mut self) -> Result<Vec<ble::Peripheral>> {
        Ok(self
            .searcher
            .search(&proto::UUID_SERVICE, self.timeout)
            .await
            .context("Error on searching cubes")?)
    }
}
