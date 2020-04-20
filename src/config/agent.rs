//! `sagan.toml` monitoring agent configuration settings

use crate::error::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tendermint::{config::TendermintConfig, net};


/// Tendermint node-related config settings from `sagan.toml`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    /// Location of monitored Tendermint node's `--home` directory
    pub node_home: Option<PathBuf>,

    /// Location of collector
    pub collector: CollectorAddr,

    /// Event query to collect via the Tendermint web socket
    pub query:String,

    /// URL for the Tendermint RPC port to connect to
    pub rpc:tendermint::net::Address,
}

impl AgentConfig {
    /// Path to the node's configuration directory
    pub fn config_dir(&self) -> Option<PathBuf> {
     match self.node_home{
            Some(ref path) =>Some(path.join("config")),
            None => None,
        }
    }

    /// Path to the node's `config.toml` file
    pub fn config_toml_path(&self) -> Option<PathBuf> {
        match self.config_dir(){
        Some(path) => Some(path.join("config.toml")),
        None => None,
        }
    }

    /// Load `TendermintConfig` using this node configuration
    pub fn load_tendermint_config(&self) -> Result<Option<TendermintConfig>, Error> {
        match &self.config_toml_path(){
            Some(path) =>Ok(Some(TendermintConfig::load_toml_file(path).map_err(|e| format_err!(ErrorKind::ConfigError, "{}", e))?)),
            None => Ok(None),
        }
    }
}

/// Collector config
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum CollectorAddr {
    /// Collector HTTP config
    #[serde(rename = "http")]
    Http(HttpConfig),
}

/// Http config
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    /// Address of collector http service
    pub addr: net::Address,
}
