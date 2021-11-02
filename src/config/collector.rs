//! `cosmon.toml` Collector configuration settings

use serde::{Deserialize, Serialize};

use crate::config::network;

pub mod listen;

/// Collector config settings from `cosmon.toml`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Listen configuration
    pub listen: listen::Config,

    /// Networks this collector is collecting information about
    pub networks: network::Config,
}
