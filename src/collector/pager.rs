//! Collector pager

use crate::{collector, config, prelude::*};
use std::time::Duration;
use std::time::SystemTime;
use tokio::time;
use tower::{Service, ServiceExt};
use futures::future;

/// The collector's [`Pager`] collects pageable events
pub struct Pager {
    /// Interval at which to poll
    poll_interval: Duration,

    ///Last sent page event to Datadog then forwarded to Pagerduty
    last_paged_at: Option<SystemTime>,
}

impl Pager {
    pub fn new(config: &config::collector::Config) -> Result<Self, Error> {
        let now = SystemTime::now();

        Ok(Self {
            poll_interval: Duration::from_secs(1),
            last_paged_at: None,
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
        let mut interval = time::interval(self.poll_interval);

        loop {
            interval.tick().await;
            self.poll(&collector).await;
            info!("waiting for {:?}", self.poll_interval);
        }
    }

    /// Poll sources.
    #[cfg_attr(not(feature = "mintscan"), allow(unused_variables))]
    async fn poll<S>(&self, mut collector: &S)
        where
            S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Clone
            + 'static,
    {
        let mut interval = time::interval(self.poll_interval);
        loop {
            interval.tick().await;

            collector
                .ready()
                .await
                .expect("collector not ready")
                .call(
                    collector::request::GetPageEvents {}
                        .into(),
                )
                .await
                .expect("error sending poller info");
        }
    }

}