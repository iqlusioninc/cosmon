//! `sagan.toml` Collector configuration settings

use serde::{Deserialize, Serialize};
use tendermint::net;

/// Collector config settings from `sagan.toml`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorConfig {
    /// Address to bind to
    pub listen_addr: net::Address,
}
