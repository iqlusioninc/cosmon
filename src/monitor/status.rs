//! Node status monitoring

use super::message::Message;
use crate::error::Error;
use serde::{Deserialize, Serialize};
pub use tendermint::rpc::{self, endpoint::status::SyncInfo};

/// Node status monitor: monitors the `/status` RPC endpoint.
///
/// Monitors current chain, node, and validator status.
#[derive(Clone, Debug)]
pub struct Status {
    /// Last chain status
    pub chain: ChainStatus,

    /// Last node status
    pub node: tendermint::node::Info,

    /// Last validator status
    pub validator: tendermint::validator::Info,
}

impl Status {
    /// Create a new `/status` endpoint monitor
    pub fn new(rpc_client: &rpc::Client) -> Result<Self, Error> {
        Ok(Self::from(rpc_client.status()?))
    }

    /// Update internal state using the given RPC client, returning any changes
    pub fn update(&mut self, rpc_client: &rpc::Client, force: bool) -> Result<Vec<Message>, Error> {
        let status = rpc_client.status()?;
        let mut output = vec![];

        let chain_status = ChainStatus(status.sync_info);
        if chain_status != self.chain || force {
            self.chain = chain_status.clone();
            output.push(chain_status.into());
        }

        if status.node_info != self.node || force {
            self.node = status.node_info.clone();
            output.push(status.node_info.into());
        }

        if status.validator_info != self.validator || force {
            self.validator = status.validator_info.clone();
            output.push(status.validator_info.into());
        }

        Ok(output)
    }
}

impl From<rpc::endpoint::status::Response> for Status {
    fn from(response: rpc::endpoint::status::Response) -> Self {
        Self {
            chain: response.sync_info.into(),
            node: response.node_info,
            validator: response.validator_info,
        }
    }
}

/// Chain status info
// TODO(tarcieri): derive `Eq/PartialEq` on upstream `tendermint::SyncInfo`
// Then this type can go away as it's just here for that
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChainStatus(SyncInfo);

impl PartialEq for ChainStatus {
    fn eq(&self, other: &Self) -> bool {
        let a = &self.0;
        let b = &other.0;

        a.latest_block_hash == b.latest_block_hash
            && a.latest_app_hash == b.latest_app_hash
            && a.latest_block_height == b.latest_block_height
            && a.latest_block_time == b.latest_block_time
            && a.catching_up == b.catching_up
    }
}

impl Eq for ChainStatus {}

impl From<SyncInfo> for ChainStatus {
    fn from(sync_info: SyncInfo) -> ChainStatus {
        ChainStatus(sync_info)
    }
}
