//! `sagan.toml` configuration file

pub mod agent;

use self::agent::AgentConfig;
use abscissa_core::Config;
use serde::{Deserialize, Serialize};

/// `sagan.toml` configuration settings
#[derive(Clone, Config, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SaganConfig {
    /// Monitoring agent configuration
    pub agent: Option<AgentConfig>,
}

impl SaganConfig {
    /// Are we configured to be an agent?
    pub fn is_agent(&self) -> bool {
        self.agent.is_some()
    }
}

impl Default for SaganConfig {
    fn default() -> Self {
        Self { agent: None }
    }
}
