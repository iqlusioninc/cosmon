//! Collector state

use crate::{
    config::collector::CollectorConfig,
    message,
    network::{self, Network},
    prelude::*,
};
use std::{collections::BTreeMap as Map, process};

/// Collector state
#[derive(Debug)]
pub struct State {
    /// Network states
    networks: Map<network::Id, Network>,
}

impl State {
    /// Initialize collector state
    pub fn new(config: &CollectorConfig) -> Result<Self, Error> {
        let mut networks = Map::default();

        for network in Network::from_config(&config.networks) {
            let network_id = network.id();

            if networks.insert(network_id.clone(), network).is_some() {
                // TODO(tarcieri): bubble up this error
                status_err!("duplicate networks in config: {}", &network_id);
                process::exit(1);
            }
        }

        Ok(Self { networks })
    }

    /// Borrow a network registered with this application
    pub fn network(&self, network_id: &network::Id) -> Option<&Network> {
        self.networks.get(network_id)
    }

    /// Handle an incoming status message from a monitor
    pub fn handle_message(&mut self, message: message::Envelope) {
        if let Some(network) = self.networks.get_mut(&message.network.clone().into()) {
            network.handle_message(message);
        } else {
            // TODO(tarcieri): bubble up this error?
            warn!("got message for unregistered network: {}", &message.network);
        }
    }
}
