//! `sagan.toml` configuration settings for the supervised Tendermint node

use super::tendermint::TendermintConfig;
use abscissa_core::{Config, FrameworkError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Tendermint node-related config settings from `sagan.toml`
#[derive(Clone, Config, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    /// Directory for config and data (i.e. argument to `--home`)
    pub home: PathBuf,
}

impl NodeConfig {
    /// Path to the node's configuration directory
    pub fn config_dir(&self) -> PathBuf {
        self.home.join("config")
    }

    /// Path to the Node's `config.toml` file
    pub fn config_toml_path(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }

    /// Load `TendermintConfig` using this node configuration
    pub fn load_tendermint_config(&self) -> Result<TendermintConfig, FrameworkError> {
        TendermintConfig::load_toml_file(&self.config_toml_path())
    }
}
