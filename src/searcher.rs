use crate::{
    ble::{self, PeripheralOps, SearchOps},
    proto, Cube,
};
use anyhow::{anyhow, Context, Result};

pub struct Searcher {
    searcher: ble::Searcher,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            searcher: ble::Searcher::new(),
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

    /// Find the nearest cube.
    pub async fn nearest(&mut self) -> Result<Cube> {
        self.do_search()
            .await?
            .into_iter()
            .max_by(|a, b| a.rssi().cmp(&b.rssi()))
            .map(|a| Cube::new(a))
            .ok_or_else(|| anyhow!("No cube found"))
    }

    async fn do_search(&mut self) -> Result<Vec<ble::Adaptor>> {
        Ok(self
            .searcher
            .search(&proto::UUID_SERVICE)
            .await
            .context("Error on searching cubes")?)
    }
}
