//! Networks that agents are actively monitoring

mod id;
pub mod tendermint;

pub use self::id::Id;
use crate::{config::collector::NetworkConfig, message};
use serde::Serialize;

/// Types of networks
#[derive(Debug)]
pub enum Network {
    /// Tendermint networks
    Tendermint(Box<self::tendermint::Network>),
}

impl Network {
    /// Initialize networks from the given configuration
    pub fn from_config(config: &NetworkConfig) -> Vec<Network> {
        let mut networks = vec![];

        for id in &config.tendermint {
            networks.push(Network::Tendermint(Box::new(
                self::tendermint::Network::new(id.clone()),
            )))
        }

        networks
    }

    /// Get a network's ID
    pub fn id(&self) -> Id {
        match self {
            Network::Tendermint(tm) => tm.id(),
        }
    }

    /// Handle an incoming status message from a monitor
    pub fn handle_message(&mut self, envelope: message::Envelope) {
        match self {
            Network::Tendermint(tm) => tm.handle_message(envelope),
        }
    }

    /// Return JSON serialization of this network's information
    pub fn state(&self) -> impl Serialize {
        match self {
            Network::Tendermint(tm) => tm.state(),
        }
    }
}
