use crate::{
    ble::{self, PeripheralOps},
    proto, Cube,
};
use anyhow::{anyhow, Context, Result};
use std::fmt::{self, Debug};
use std::time::Duration;

const SEARCH_TIMEOUT: Duration = Duration::from_secs(3);

/// Searcher to search cubes.
pub struct Searcher {
    searcher: ble::Searcher,
}

impl Debug for Searcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Searcher").finish()
    }
}

impl Searcher {
    /// Creates a new searcher instance.
    ///
    /// The default search timeout is 3 seconds.
    /// Use [`Searcher::new_with_timeout`][] to specify custom timeout.
    pub fn new() -> Self {
        Self {
            searcher: ble::searcher(),
        }
    }

    /// Searches for all cubes.
    ///
    /// This searches for cubes for 3 seconds.
    /// Use [`Searcher::all_timeout`] to use custom timeout.
    pub async fn all(&mut self) -> Result<Vec<Cube>> {
        self.all_timeout(SEARCH_TIMEOUT).await
    }

    /// Finds the nearest cube.
    ///
    /// This searches for cubes for 3 seconds.
    /// Use [`Searcher::nearest_timeout`] to use custom timeout.
    pub async fn nearest(&mut self) -> Result<Cube> {
        self.nearest_timeout(SEARCH_TIMEOUT).await
    }

    /// Searches for all cubes with custom timeout.
    ///
    /// Cubes are sorted from nearest to farest.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let cubes = Cube::search().all_timeout(Duration::from_secs(8)).await.unwrap();
    /// }
    /// ```
    pub async fn all_timeout(&mut self, timeout: Duration) -> Result<Vec<Cube>> {
        let mut cubes: Vec<_> = self
            .do_search(timeout)
            .await?
            .into_iter()
            .map(|a| Cube::new(a))
            .collect();
        cubes.sort_by(|a, b| b.rssi().cmp(&a.rssi()));
        Ok(cubes)
    }

    /// Finds the nearest cube with custom timeout.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use toio::Cube;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let cube = Cube::search().nearest_timeout(Duration::from_secs(8)).await.unwrap();
    /// }
    /// ```
    pub async fn nearest_timeout(&mut self, timeout: Duration) -> Result<Cube> {
        self.do_search(timeout)
            .await?
            .into_iter()
            .max_by(|a, b| a.rssi().cmp(&b.rssi()))
            .map(|a| Cube::new(a))
            .ok_or_else(|| anyhow!("No cube found"))
    }

    async fn do_search(&mut self, timeout: Duration) -> Result<Vec<ble::Peripheral>> {
        Ok(self
            .searcher
            .search(&proto::UUID_SERVICE, timeout)
            .await
            .context("Error on searching cubes")?)
    }
}
