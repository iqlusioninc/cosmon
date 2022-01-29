//! Networks that agents are actively monitoring

pub mod tendermint;

mod id;
mod state;

pub use self::{id::Id, state::State};
use crate::{collector, config, message};

/// Types of networks.
#[derive(Debug, Clone)]
pub enum Network {
    /// Tendermint network
    Tendermint(Box<self::tendermint::Network>),
}

impl Network {
    /// Initialize network from the given configuration.
    pub fn from_config(config: &config::network::Config) -> Vec<Network> {
        let mut networks = vec![];

        for tm_config in &config.tendermint {
            networks.push(Network::Tendermint(Box::new(
                self::tendermint::Network::new(tm_config.chain_id.clone()),
            )))
        }

        networks
    }

    /// Get a network's ID.
    pub fn id(&self) -> Id {
        match self {
            Network::Tendermint(tm) => tm.id(),
        }
    }

    /// Handle an incoming status message from a monitor.
    pub fn handle_message(&mut self, envelope: message::Envelope) {
        match self {
            Network::Tendermint(tm) => tm.handle_message(envelope),
        }
    }

    /// Handle an incoming pager info message from a monitor.
    pub async fn handle_poll_event(&mut self, poll_event: collector::PollEvent) {
        match self {
            Network::Tendermint(tm) => tm.handle_poll_event(poll_event).await,
        };
    }

    pub fn get_pager_events(&mut self) -> Option<String> {
        todo!("")

    }

    /// Return a snapshot of the network state.
    pub fn state(&self) -> State {
        match self {
            Network::Tendermint(tm) => State::Tendermint(tm.state()),
        }
    }
}
