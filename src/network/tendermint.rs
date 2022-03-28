//! Tendermint network types

use super::Id;
use crate::{
    collector::PollEvent,
    message::{Envelope, Message},
    monitor::{net_info::Peer, status::ChainStatus},
    prelude::*,
};
use serde::Serialize;
use std::time::{Duration, SystemTime};

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

    /// Page events
    page: Vec<String>,

    ///Last sent page event to Datadog then forwarded to Pagerduty
    last_paged_at: Option<SystemTime>,
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
            page: vec![],
            last_paged_at: None,
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
    pub fn handle_poll_event(&mut self, poll_event: PollEvent) {
        dbg!(&poll_event);
        let missed_blocks = poll_event.missed_blocks.unwrap();
        // todo add page_threshold to config
        let page_threshold = 10;
        if missed_blocks > page_threshold {
            self.page.push(format!(
                "'{}' missed {} blocks!",
                poll_event.network_id, missed_blocks
            ));
        }
    }

    /// Get page events set by `PAGE_INTERVAL`
    pub fn get_page_event(&mut self) -> Option<String> {
        const PAGE_INTERVAL: Duration = Duration::from_secs(10 * 60);

        if let Some(page) = self.page.pop() {
            if let Some(last_paged_at) = self.last_paged_at {
                if SystemTime::now().duration_since(last_paged_at).unwrap() < PAGE_INTERVAL {
                    return None;
                }
            }

            self.last_paged_at = Some(SystemTime::now());
            return Some(page);
        }

        None
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
