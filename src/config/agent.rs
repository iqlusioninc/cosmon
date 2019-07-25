//! `sagan.toml` monitoring agent configuration settings

use abscissa_core::{FrameworkError, FrameworkErrorKind::ConfigError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tendermint::config::TendermintConfig;

/// Tendermint node-related config settings from `sagan.toml`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    /// Location of monitored Tendermint node's `--home` directory
    pub node_home: PathBuf,
}

impl AgentConfig {
    /// Path to the node's configuration directory
    pub fn config_dir(&self) -> PathBuf {
        self.node_home.join("config")
    }

    /// Path to the node's `config.toml` file
    pub fn config_toml_path(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }

    /// Load `TendermintConfig` using this node configuration
    pub fn load_tendermint_config(&self) -> Result<TendermintConfig, FrameworkError> {
        Ok(TendermintConfig::load_toml_file(&self.config_toml_path())
            .map_err(|e| err!(ConfigError, "{}", e))?)
    }
}
