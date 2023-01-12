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
    pub mintscan: Option<MintscanConfig>,

    /// Explorers Guru API endpoint.
    pub ngexplorers: Option<NgExplorersConfig>,
}

/// Mintscan configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MintscanConfig {
    /// API host (e.g. `api.cosmostation.io`)
    pub host: String,

    /// Network name (e.g. `cosmos`)
    pub network: String,
}

/// Ng Explorers configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NgExplorersConfig {
    /// API host (e.g. `agoric.api.explorers.guru`)
    pub host: String,
}
