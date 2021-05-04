//! Network configuration.

pub mod tendermint;

use serde::{Deserialize, Serialize};

/// Types of network this collector is collecting information about
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Tendermint network
    #[serde(default)]
    pub tendermint: Vec<tendermint::Config>,
}
