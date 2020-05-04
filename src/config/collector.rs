//! `sagan.toml` Collector configuration settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tendermint::net;

/// Collector config settings from `sagan.toml`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorConfig {
    /// Address to bind to
    pub listen_addr: net::Address,

    /// Networks this collector is collecting information about
    pub networks: NetworkConfig,

    /// Host ip for the StatsD Deserver
    pub statsd: String,

    /// Map from Channel ID to team
    pub teamchannels: Option<HashMap<String, String>>,

    /// Map from Address to team
    pub teamaddresses: Option<HashMap<String, String>>,

    ///Prefix for metrics
    pub prefix: String,
}

/// Types of networks this collector is collecting information about
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Tendermint networks
    #[serde(default)]
    pub tendermint: Vec<tendermint::chain::Id>,
}
