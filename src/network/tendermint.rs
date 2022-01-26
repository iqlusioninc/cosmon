//! Tendermint network types


use datadog;
use datadog::{send_stream_event, StreamEvent};
use hostname;
use super::Id;
use crate::{
    message::{Envelope, Message},
    monitor::{net_info::Peer, status::ChainStatus},
    prelude::*,
    collector::PollEvent
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::time::SystemTime;

/// Tendermint network
#[derive(Debug, Clone)]
pub struct Network {
    /// Chain ID for this network
    id: tendermint::chain::Id,

    /// Nodes in this network
    nodes: Map<tendermint::node::Id, Node>,

    /// Network peers
    peers: Vec<Peer>,

    /// Chain status (if known)
    chain: Option<ChainStatus>,

    /// Validators
    validators: Option<tendermint::validator::Info>,
}

impl Network {
    /// Create new Tendermint network state
    pub fn new(id: tendermint::chain::Id) -> Self {
        Self {
            id,
            nodes: Map::new(),
            peers: vec![],
            chain: None,
            validators: None,
        }
    }

    /// Get this network's ID
    pub fn id(&self) -> Id {
        self.id.clone().into()
    }

    /// Serialize information about this network as JSON
    pub fn state(&self) -> State {
        State::new(self)
    }

    /// Update internal state from incoming messages
    pub fn handle_message(&mut self, envelope: Envelope) {
        if envelope.network != self.id {
            return;
        }

        // Extract node information in advance
        for msg in &envelope.msg {
            match msg {
                Message::Node(ref node_info) => self.update_node(node_info),
                Message::Peers(ref peer_info) => self.update_peer(peer_info),
                Message::Chain(ref chain_info) => self.update_chain(chain_info),
                Message::Validator(ref validator_info) => self.update_validator(validator_info),
            }
        }
    }

    /// Handle incoming poll event
    pub async fn handle_poll_event(&mut self, poll_event: PollEvent) {
        dbg!(&poll_event);
        let last_signed_height = poll_event.last_signed_height.unwrap();
        let page_threshold = last_signed_height + 20;
        let current_height = poll_event.current_height;
        if current_height > page_threshold {
            let dd_api_key = env::var("DD_API_KEY").unwrap();
            let hostname = hostname::get().unwrap();
            let mut ddtags = BTreeMap::new();
            ddtags.insert("env".to_owned(), "staging".to_owned());
            let stream_event = StreamEvent {
                aggregation_key: None,
                alert_type: Some(datadog::AlertType::Error),
                date_happened: Some(SystemTime::now()),
                device_name: None,
                hostname: Some(hostname.to_string_lossy().to_string()),
                priority: Some(datadog::Priority::Normal),
                related_event_id: None,
                tags: Some(ddtags),
                // Text field must contain @pagerduty to trigger alert
                text: format!("@pagerduty cosmon poll event: {:?}", &poll_event),
                title: "cosmon poll event".to_owned(),
            };

            // send stream event to datadog which forwards to pagerduty
            let stream_event = send_stream_event(&stream_event, dd_api_key).await;
            match stream_event {
                Ok(()) => {
                    dbg!("event sent to datadog");
                }
                Err(_err) => {
                    warn!("unable to sent event to datadog");
                }
            }
        }

    }

    /// Update information about a particular node
    fn update_node(&mut self, node_info: &tendermint::node::Info) {
        info!(
            "got node status update from: {} (moniker: {})",
            &node_info.id, &node_info.moniker
        );

        if let Some(_node) = self.nodes.get(&node_info.id) {
            // TODO(tarcieri): update existing node information
        } else {
            let node = Node::from(node_info);
            self.nodes.insert(node.id, node);
        }
    }

    /// Update information about peers
    fn update_peer(&mut self, peer_info: &[Peer]) {
        info!("peers update: {:?} ", peer_info);
        self.peers = peer_info.to_vec();
    }

    /// Update information about chain status
    fn update_chain(&mut self, chain_info: &ChainStatus) {
        info!("chain status update: {:?}", chain_info);
        self.chain = Some(chain_info.clone());
    }

    /// Update information about validators
    fn update_validator(&mut self, validator_info: &tendermint::validator::Info) {
        info!("validator update: {:?}", validator_info);
        self.validators = Some(validator_info.clone());
    }
}

/// Nodes in Tendermint network
#[derive(Clone, Debug, Serialize)]
pub struct Node {
    /// Node ID
    pub id: tendermint::node::Id,

    /// Node moniker
    pub moniker: tendermint::Moniker,
}

impl<'a> From<&'a tendermint::node::Info> for Node {
    fn from(node_info: &'a tendermint::node::Info) -> Node {
        Node {
            id: node_info.id,
            moniker: node_info.moniker.clone(),
        }
    }
}

/// Snapshot of current Tendermint network state
#[derive(Debug, Serialize)]
pub struct State {
    nodes: Vec<Node>,
    peers: Vec<Peer>,
    chain: Option<ChainStatus>,
    validators: Option<tendermint::validator::Info>,
}

impl State {
    fn new(network: &Network) -> Self {
        Self {
            nodes: network.nodes.values().cloned().collect(),
            peers: network.peers.clone(),
            chain: network.chain.clone(),
            validators: network.validators.clone(),
        }
    }
}
