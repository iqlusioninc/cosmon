//! Message types (sent to collector)

use crate::monitor::{net_info::Peer, status::ChainStatus};
use abscissa_core::time::{DateTime, Utc};
use relayer_modules::events::IBCEvent;
use serde::{Deserialize, Serialize};
use tendermint::{chain, node};

/// Every event reported to the collector is a sequence of messages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    /// Chain synchronization status for a node
    #[serde(rename = "chain")]
    Chain(Box<ChainStatus>),

    /// Node information
    #[serde(rename = "node")]
    Node(Box<tendermint::node::Info>),

    /// Validator information
    #[serde(rename = "validator")]
    Validator(Box<tendermint::validator::Info>),

    /// Peer connections
    #[serde(rename = "peers")]
    Peers(Vec<Peer>),
    /// CreateClient IBC event
    #[serde(rename = "event_ibc")]
    EventIBC(IBCEvent),
}

impl From<ChainStatus> for Message {
    fn from(chain_status: ChainStatus) -> Message {
        Message::Chain(Box::new(chain_status))
    }
}

impl From<tendermint::node::Info> for Message {
    fn from(node_info: tendermint::node::Info) -> Message {
        Message::Node(Box::new(node_info))
    }
}

impl From<tendermint::validator::Info> for Message {
    fn from(validator_info: tendermint::validator::Info) -> Message {
        Message::Validator(Box::new(validator_info))
    }
}

impl From<Vec<Peer>> for Message {
    fn from(peers: Vec<Peer>) -> Message {
        Message::Peers(peers)
    }
}

impl From<IBCEvent> for Message {
    fn from(event: IBCEvent) -> Message {
        Message::EventIBC(event)
    }
}

/// Message envelope - contains information about the node events are
/// originating from.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Envelope {
    /// Chain ID reporting in
    pub network: chain::Id,

    /// Node ID reporting in
    pub node: node::Id,

    /// Timestamp when this message envelope was created
    pub ts: DateTime<Utc>,

    /// Messages inside of the envelope
    pub msg: Vec<Message>,
}

impl Envelope {
    /// Create a new message envelope from the given messages
    pub fn new(network: chain::Id, node_id: node::Id, msg: Vec<Message>) -> Option<Envelope> {
        if msg.is_empty() {
            None
        } else {
            Some(Self {
                network,
                node: node_id,
                ts: Utc::now(),
                msg,
            })
        }
    }

    /// Serialize this message envelope as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
