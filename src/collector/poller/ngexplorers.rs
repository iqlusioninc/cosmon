//! NgExplorers poller

use crate::{collector, config, error::Error, network, prelude::*};
use iqhttp::{HttpsClient, Query};
use serde::Deserialize;
use tendermint::chain;
use tower::{util::ServiceExt, Service};

/// Ng Explorers poller
pub struct Poller {
    /// API hostname
    host: String,

    /// NgExplorers client
    client: HttpsClient,

    /// Tendermint chain ID
    chain_id: chain::Id,

    /// Validator operator address (if configured)
    validator_addr: Option<String>,
}

impl Poller {
    /// Name of this poller source
    pub const SOURCE_NAME: &'static str = "ngexplorers";

    /// Create a new NgExplorers poller for the given Tendermint network, if it
    /// has a NgExplorers configuration.
    pub fn new(config: &config::network::tendermint::Config) -> Option<Self> {
        config.ngexplorers.as_ref().map(|ngexplorers_config| {
            let host = ngexplorers_config.host.clone();
            let client = HttpsClient::new(&host);

            Self {
                host,
                client,
                chain_id: config.chain_id.clone(),
                validator_addr: config.validator_addr.clone(),
            }
        })
    }

    /// Get `api/blocks/uptime` endpoint.
    ///
    /// Accepts account address for the validator.
    pub async fn validator_uptime(&self, addr: &str) -> Result<Vec<Block>, Error> {
        let path = format!("/api/blocks/uptime/{}", addr);
        let mut query = Query::new();
        query.add("count", "100");

        Ok(self.client.get_json(&path, &query).await?)
    }

    /// Poll NgExplorers for status and validator info
    pub async fn poll<S>(&self, mut collector: S)
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Clone
            + 'static,
    {
        if let Some(addr) = &self.validator_addr {
            match self.validator_uptime(addr).await {
                Ok(uptime) => {
                    let mut missed_blocks = 0;

                    for block in uptime {
                        if !block.signed {
                            missed_blocks += 1;
                        }
                    }

                    collector
                        .ready()
                        .await
                        .expect("collector not ready")
                        .call(
                            collector::request::PollEvent {
                                source: Self::SOURCE_NAME,
                                network_id: network::Id::from(&self.chain_id),
                                missed_blocks: Some(missed_blocks),
                            }
                            .into(),
                        )
                        .await
                        .expect("error sending poller info");
                }
                Err(err) => {
                    warn!(
                        "[{}] can't fetch validator uptime for {} from {}: {}",
                        &self.chain_id, addr, &self.host, err
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Block {
    /// Height
    pub height: u64,

    /// Signed
    pub signed: bool,
}
