//! Network information monitor

use super::message::Message;
use crate::error::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tendermint::{net, node};
use tendermint_rpc::{Client, HttpClient};

/// Map of peer IDs to their peer information
type PeerMap = BTreeMap<node::Id, Peer>;

/// Network information monitor: monitors the `/net_info` endpoint.
#[derive(Clone, Debug)]
pub struct NetInfo {
    /// Persistent peers
    persistent_peers: Vec<net::Address>,

    /// Private peer IDs
    private_peer_ids: Vec<node::Id>,

    /// Previous peer list
    peers: Vec<Peer>,
}

impl NetInfo {
    /// Create a new network information monitor
    pub fn new(persistent_peers: Vec<net::Address>, private_peer_ids: Vec<node::Id>) -> Self {
        Self {
            persistent_peers,
            private_peer_ids,
            peers: vec![],
        }
    }

    /// Update internal state using the given RPC client, returning any changes
    // TODO(tarcieri): don't error out on attacker-controlled values; log instead
    pub async fn update(
        &mut self,
        rpc_client: &HttpClient,
        force: bool,
    ) -> Result<Vec<Message>, Error> {
        let mut peer_map = self.peer_map()?;

        for peer_info in rpc_client.net_info().await?.peers {
            let node_id = &peer_info.node_info.id;
            let listen_addr = peer_info
                .node_info
                .listen_addr
                .to_net_address()
                .ok_or_else(|| {
                    format_err!(
                        ErrorKind::ConfigError,
                        "can't parse peer's listen address: {}",
                        &peer_info.node_info.listen_addr
                    )
                })?;

            if let Some(peer) = peer_map.get_mut(node_id) {
                peer.connection = if peer_info.is_outbound {
                    ConnectionStatus::Out
                } else {
                    ConnectionStatus::In
                };
            } else if let net::Address::Tcp { port, .. } = listen_addr {
                let addr = net::Address::Tcp {
                    peer_id: Some(*node_id),
                    host: peer_info.remote_ip.to_string(),
                    port,
                };

                let status = if peer_info.is_outbound {
                    ConnectionStatus::Out
                } else {
                    ConnectionStatus::In
                };

                let peer = Peer {
                    addr,
                    connection: status,
                    persistent: false,
                    private: false,
                };
                assert_eq!(peer_map.insert(*node_id, peer), None);
            } else {
                fail!(
                    ErrorKind::ConfigError,
                    "unsupported peer listen addr: {}",
                    listen_addr
                );
            }
        }

        let peers = peer_map.values().cloned().collect::<Vec<_>>();

        let mut output = vec![];

        if peers != self.peers || force {
            self.peers = peers.clone();
            output.push(peers.into());
        }

        Ok(output)
    }

    /// Create peer ID map from the configuration values
    fn peer_map(&self) -> Result<PeerMap, Error> {
        let mut map = PeerMap::new();

        for addr in &self.persistent_peers {
            if let net::Address::Tcp {
                peer_id: Some(id), ..
            } = addr
            {
                let peer = Peer {
                    addr: addr.clone(),
                    connection: ConnectionStatus::None,
                    persistent: true,
                    private: false,
                };

                if map.insert(*id, peer).is_some() {
                    fail!(
                        ErrorKind::ConfigError,
                        "duplicate persistent_peer node ID: {}",
                        id
                    );
                }
            } else {
                fail!(
                    ErrorKind::ConfigError,
                    "unsupported persistent_peer addr: {}",
                    addr
                );
            }
        }

        for id in &self.private_peer_ids {
            if let Some(peer) = map.get_mut(id) {
                peer.private = true;
            } else {
                fail!(ErrorKind::ConfigError, "unkown private_peer_id: {}", id);
            }
        }

        Ok(map)
    }
}

/// Information about a specific network peer
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Peer {
    /// Address of the remote peer
    pub addr: net::Address,

    /// Connection status
    pub connection: ConnectionStatus,

    /// Is this peer supposed to be persistently connected
    pub persistent: bool,

    /// Is this peer marked as being private?
    pub private: bool,
}

/// Status of the connection
#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConnectionStatus {
    /// Connected outbound
    #[serde(rename = "out")]
    Out,

    /// Inbound connection received
    #[serde(rename = "in")]
    In,

    /// Disconnected from this peer
    #[serde(rename = "none")]
    None,
}

impl ConnectionStatus {
    /// Are we connected?
    pub fn is_connected(self) -> bool {
        match self {
            ConnectionStatus::Out | ConnectionStatus::In => true,
            ConnectionStatus::None => false,
        }
    }
}
