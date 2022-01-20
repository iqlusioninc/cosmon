//! Mintscan poller

use crate::{collector, config, network, prelude::*};
use datadog;
use datadog::{send_stream_event, StreamEvent};
use hostname;
use mintscan::{Address, Mintscan};
use std::collections::BTreeMap;
use std::env;
use std::time::SystemTime;
use tendermint::chain;
use tower::{util::ServiceExt, Service};

/// Mintscan poller
pub struct Poller {
    /// API hostname
    host: String,

    /// Mintscan client
    client: Mintscan,

    /// Tendermint chain ID
    chain_id: chain::Id,

    /// Validator operator address (if configured)
    validator_addr: Option<Address>,
}

impl Poller {
    /// Name of this poller source
    pub const SOURCE_NAME: &'static str = "mintscan";

    /// Create a new Mintscan poller for the given Tendermint network, if it
    /// has a Mintscan configuration.
    pub fn new(config: &config::network::tendermint::Config) -> Option<Self> {
        config.mintscan.as_ref().map(|mintscan_config| {
            let host = mintscan_config.host.clone();
            let client = Mintscan::new(&host);

            Self {
                host,
                client,
                chain_id: config.chain_id.clone(),
                validator_addr: config.validator_addr.clone(),
            }
        })
    }

    /// Poll Mintscan for status and validator info
    pub async fn poll<S>(&self, mut collector: S)
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Clone
            + 'static,
    {
        let current_height = match self.client.status().await {
            Ok(status) => status.block_height.into(),
            Err(err) => {
                warn!("[{}] error polling {}: {}", &self.chain_id, &self.host, err);
                return;
            }
        };

        let mut last_signed_height = None;

        if let Some(addr) = &self.validator_addr {
            match self.client.validator_uptime(addr).await {
                Ok(uptime) => {
                    dbg!(&uptime);
                    last_signed_height = Some(uptime.latest_height.into());
                    dbg!(uptime.uptime.len());
                    if uptime.uptime.len() >= 2 {
                        let dd_api_key = env::var("DD_API_KEY").unwrap();
                        let hostname = hostname::get().unwrap();
                        let mut ddtags = BTreeMap::new();
                        ddtags.insert("env".to_owned(), "staging".to_owned());
                        let stream_event = StreamEvent {
                            aggregation_key: None,
                            alert_type: Some(datadog::AlertType::Error),
                            date_happened: Some(SystemTime::now()),
                            device_name: None,
                            hostname: Some(hostname.to_string_lossy().to_string()),
                            priority: Some(datadog::Priority::Normal),
                            related_event_id: None,
                            tags: Some(ddtags),
                            // Text field must contain @pagerduty to trigger alert
                            text: format!("@pagerduty missed blocks alert: {:?}", &uptime),
                            title: "missed blocks alert test".to_owned(),
                        };

                        // send stream event to datadog which forwards to pagerduty
                        let stream_event = send_stream_event(&stream_event, dd_api_key).await;
                        match stream_event {
                            Ok(()) => {
                                dbg!("event sent to datadog");
                            }
                            Err(_err) => {
                                warn!("unable to sent event to datadog");
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        "[{}] can't fetch validator uptime for {} from {}: {}",
                        &self.chain_id, addr, &self.host, err
                    );
                    return;
                }
            };
        }

        collector
            .ready()
            .await
            .expect("collector not ready")
            .call(
                collector::request::PollEvent {
                    source: Self::SOURCE_NAME,
                    network_id: network::Id::from(&self.chain_id),
                    current_height,
                    last_signed_height,
                }
                .into(),
            )
            .await
            .expect("error sending poller info");
    }
}
