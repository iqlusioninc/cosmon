//! Tendermint network types

use super::Id;
use crate::{
    message::{Envelope, Message},
    monitor::{net_info::Peer, status::ChainStatus},
    prelude::*,
};
use serde::Serialize;
use std::collections::BTreeMap as Map;

/// Tendermint networks
#[derive(Debug)]
pub struct Network {
    /// Chain ID for this network
    id: tendermint::chain::Id,

    /// Nodes in this network
    nodes: Map<tendermint::node::Id, Node>,

    peers: Vec<Peer>,

    chain: Option<ChainStatus>,

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
        self.id.into()
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

        self.peers = peer_info.to_vec().clone();
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

/// Nodes in Tendermint networks
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

/// Snapshot of current network state
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
            nodes: network.nodes.iter().map(|(_, node)| node.clone()).collect(),
            peers: network.peers.clone(),
            chain: network.chain.clone(),
            validators: network.validators.clone(),
        }
    }
}
