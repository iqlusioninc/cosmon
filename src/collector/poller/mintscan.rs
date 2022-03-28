//! Mintscan poller

use crate::{collector, config, network, prelude::*};
use mintscan::{Address, Mintscan};
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

        let mut missed_blocks = None;
        if let Some(addr) = &self.validator_addr {
            match self.client.validator_uptime(addr).await {
                Ok(uptime) => {
                    dbg!(&uptime);
                    missed_blocks = Some(uptime.uptime.len());

                    dbg!(uptime.uptime.len());
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
                    missed_blocks: missed_blocks,
                }
                .into(),
            )
            .await
            .expect("error sending poller info");
    }
}
