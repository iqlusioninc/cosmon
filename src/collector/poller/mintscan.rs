//! Mintscan poller

use crate::{config, prelude::*};
use mintscan::{Address, Mintscan};
use tendermint::chain;

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

    /// Alerting threshold for missed blocks
    missed_block_threshold: u64,
}

impl Poller {
    /// Create a new Mintscan poller for the given Tendermint network, if it
    /// has a Mintscan configuration.
    pub fn new(config: &config::network::tendermint::Config) -> Option<Self> {
        config.mintscan.as_ref().map(|mintscan_config| {
            let host = mintscan_config.host.clone();
            let client = Mintscan::new(&host);

            // TODO(tarcieri): make this configurable
            let missed_block_threshold = 50;

            Self {
                host,
                client,
                chain_id: config.chain_id.clone(),
                validator_addr: config.validator_addr.clone(),
                missed_block_threshold,
            }
        })
    }

    /// Poll Mintscan for status and validator info
    pub async fn poll(&self) {
        let chain_height = match self.client.status().await {
            Ok(status) => status.block_height,
            Err(err) => {
                warn!("[{}] error polling {}: {}", &self.chain_id, &self.host, err);
                return;
            }
        };

        info!("[{}] chain height: {}", &self.chain_id, chain_height);

        if let Some(addr) = &self.validator_addr {
            let validator_height = match self.client.validator_uptime(addr).await {
                // TODO(tarcieri): do something with `uptime.uptime` (i.e. missed blocks)?
                Ok(uptime) => uptime.latest_height,
                Err(err) => {
                    warn!(
                        "[{}] can't fetch validator uptime for {} from {}: {}",
                        &self.chain_id, addr, &self.host, err
                    );
                    return;
                }
            };

            info!(
                "[{}] validator height for {}: {}",
                &self.chain_id, addr, validator_height
            );

            let height_delta = chain_height
                .value()
                .saturating_sub(validator_height.value());

            if height_delta > self.missed_block_threshold {
                error!(
                    "[{}] validator {} has missed {} blocks!",
                    &self.chain_id, addr, height_delta
                );
            }
        }
    }
}
