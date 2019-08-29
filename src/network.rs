//! Networks that agents are actively monitoring

pub mod tendermint;

use crate::{config::collector::NetworkConfig, message};
use serde::Serialize;
use std::fmt::{self, Display};

/// Network IDs
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Id(String);

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl From<::tendermint::chain::Id> for Id {
    fn from(chain_id: ::tendermint::chain::Id) -> Id {
        Id(chain_id.as_str().to_owned())
    }
}

impl From<String> for Id {
    fn from(chain_id: String) -> Id {
        Id(chain_id)
    }
}

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
                self::tendermint::Network::new(*id),
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
