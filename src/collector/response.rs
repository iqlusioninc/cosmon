//! Responses from the collector

use crate::network;

/// Responses from the collector
#[derive(Debug)]
pub enum Response {
    /// Processed a message.
    Message,

    /// Network state information.
    NetworkState(Box<network::State>),

    PagerEvents(Vec<String>),
}

impl From<network::State> for Response {
    fn from(state: network::State) -> Response {
        Response::NetworkState(Box::new(state))
    }
}

