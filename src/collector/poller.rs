//! Collector poller

#[cfg(feature = "mintscan")]
mod mintscan;

use crate::{collector, config, prelude::*};
use std::time::Duration;
use tokio::time;
use tower::Service;

#[cfg(feature = "mintscan")]
use futures::future;

/// The collector's [`Poller`] collects information from external sources
/// which aren't capable of pushing data.
pub struct Poller {
    /// Interval at which to poll
    poll_interval: Duration,

    /// Mintscan API endpoints to poll
    #[cfg(feature = "mintscan")]
    mintscan: Vec<mintscan::Poller>,
}

impl Poller {
    /// Initialize the poller from the config
    #[cfg_attr(not(feature = "mintscan"), allow(unused_variables))]
    pub fn new(config: &config::collector::Config) -> Result<Self, Error> {
        // TODO(tarcieri): configurable poll interval
        let poll_interval = Duration::from_secs(60);

        // Initialize Mintscan if configured
        #[cfg(feature = "mintscan")]
        let mintscan = config
            .networks
            .tendermint
            .iter()
            .flat_map(mintscan::Poller::new)
            .collect();

        Ok(Self {
            poll_interval,
            #[cfg(feature = "mintscan")]
            mintscan,
        })
    }

    /// Route incoming requests.
    pub async fn run<S>(self, collector: S)
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Clone
            + 'static,
    {
        if !self.has_sources() {
            info!("no sources to poll");
            return;
        }

        info!("polling every {:?}", self.poll_interval);
        let mut interval = time::interval(self.poll_interval);

        loop {
            interval.tick().await;
            self.poll(&collector).await;
            info!("waiting for {:?}", self.poll_interval);
        }
    }

    /// Poll sources.
    #[cfg_attr(not(feature = "mintscan"), allow(unused_variables))]
    async fn poll<S>(&self, collector: &S)
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Clone
            + 'static,
    {
        #[cfg(feature = "mintscan")]
        let mut futures = vec![];

        #[cfg(feature = "mintscan")]
        for mintscan_poller in &self.mintscan {
            futures.push(mintscan_poller.poll(collector.clone()));
        }

        #[cfg(feature = "mintscan")]
        future::join_all(futures).await;
    }

    /// Are there any configured sources?
    fn has_sources(&self) -> bool {
        #[cfg(feature = "mintscan")]
        if !self.mintscan.is_empty() {
            return true;
        }

        false
    }
}
