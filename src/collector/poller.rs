//! Collector poller

mod mintscan;

mod ngexplorers;

use crate::{collector, config, prelude::*};
use futures::future;
use std::time::Duration;
use tokio::time;
use tower::Service;

/// The collector's [`Poller`] collects information from external sources
/// which aren't capable of pushing data.
pub struct Poller {
    /// Interval at which to poll
    poll_interval: Duration,

    /// Mintscan API endpoints to poll
    mintscan: Vec<mintscan::Poller>,

    ngexplorers: Vec<ngexplorers::Poller>,
}

impl Poller {
    /// Initialize the poller from the config
    #[cfg_attr(not(feature = "mintscan"), allow(unused_variables))]
    pub fn new(config: &config::collector::Config) -> Result<Self, Error> {
        // TODO(tarcieri): configurable poll interval
        let poll_interval = Duration::from_secs(60);

        // Initialize Mintscan if configured
        let mintscan = config
            .networks
            .tendermint
            .iter()
            .flat_map(mintscan::Poller::new)
            .collect();

        let ngexplorers = config
            .networks
            .tendermint
            .iter()
            .flat_map(ngexplorers::Poller::new)
            .collect();

        Ok(Self {
            poll_interval,
            mintscan,
            ngexplorers,
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
        let mut mintscan_futures = vec![];

        for mintscan_poller in &self.mintscan {
            mintscan_futures.push(mintscan_poller.poll(collector.clone()));
        }

        future::join_all(mintscan_futures).await;

        let mut ngexplorers_futures = vec![];

        for ngexplorers_poller in &self.ngexplorers {
            ngexplorers_futures.push(ngexplorers_poller.poll(collector.clone()));
        }

        future::join_all(ngexplorers_futures).await;
    }

    /// Are there any configured sources?
    fn has_sources(&self) -> bool {
        if !self.mintscan.is_empty() {
            return true;
        }

        if !self.ngexplorers.is_empty() {
            return true;
        }

        false
    }
}
