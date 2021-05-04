//! Tendermint network configuration.

use serde::{Deserialize, Serialize};
use tendermint::chain;

/// Tendermint network configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Chain ID.
    pub chain_id: chain::Id,

    /// Validator operator address.
    // TODO(tarcieri): proper address type
    pub validator_addr: Option<String>,

    /// Mintscan API endpoint.
    #[cfg(feature = "mintscan")]
    pub mintscan: Option<MintscanConfig>,
}

/// Mintscan configuration.
#[cfg(feature = "mintscan")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MintscanConfig {
    /// API host (e.g. `api.cosmostation.io`)
    pub host: String,
}
