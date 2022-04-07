//! Requests to the collector

use crate::{message, network};

/// Block height type
pub type BlockHeight = u64;

/// Requests to the collector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// Handle an incoming message from an agent.
    Message(message::Envelope),

    /// Get the network state for a given network.
    NetworkState(network::Id),

    /// Get any pager events to relay to e.g. PagerDuty
    PagerEvents,

    /// Report information obtained from an external poller.
    PollEvent(PollEvent),
}

impl From<message::Envelope> for Request {
    fn from(msg: message::Envelope) -> Request {
        Request::Message(msg)
    }
}

impl From<PollEvent> for Request {
    fn from(info: PollEvent) -> Request {
        Request::PollEvent(info)
    }
}

/// Information obtained from an external poller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PollEvent {
    /// Source the data was obtained from
    // TODO(tarcieri): better type for this?
    pub source: &'static str,

    /// Network ID the information is associated with.
    pub network_id: network::Id,

    /// Current block height.
    pub current_height: BlockHeight,

    /// Last block signed by the validator for this chain, if known.
    pub missed_blocks: Option<usize>,
}
