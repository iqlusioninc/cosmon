//! Requests to the collector

use crate::{message, network};

/// Requests to the collector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// Handle an incoming message from an agent.
    Message(message::Envelope),

    /// Get the network state for a given network.
    NetworkState(network::Id),
}

impl From<message::Envelope> for Request {
    fn from(msg: message::Envelope) -> Request {
        Request::Message(msg)
    }
}
