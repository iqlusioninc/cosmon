//! `sagan.toml` configuration file

use serde::{Deserialize, Serialize};

pub mod agent;
pub mod collector;
pub mod network;

/// `sagan.toml` configuration settings
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SaganConfig {
    /// Monitoring agent configuration
    pub agent: Option<agent::Config>,

    /// Collector configuration
    pub collector: Option<collector::Config>,
}

impl SaganConfig {
    /// Are we configured to be an agent?
    pub fn is_agent(&self) -> bool {
        self.agent.is_some()
    }
}

impl Default for SaganConfig {
    fn default() -> Self {
        Self {
            agent: None,
            collector: None,
        }
    }
}
