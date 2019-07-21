//! `sagan.toml` configuration file

pub mod node;
pub mod tendermint;

pub use self::{node::NodeConfig, tendermint::TendermintConfig};
use abscissa_core::Config;
use serde::{Deserialize, Serialize};

/// `sagan.toml` configuration settings
#[derive(Clone, Config, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SaganConfig {
    /// Monitored node configuration
    pub node: NodeConfig,
}
