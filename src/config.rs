//! `sagan.toml` configuration file

pub mod agent;
pub mod collector;

use self::agent::AgentConfig;
use self::collector::CollectorConfig;
use serde::{Deserialize, Serialize};

/// `sagan.toml` configuration settings
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SaganConfig {
    /// Monitoring agent configuration
    pub agent: Option<AgentConfig>,

    /// Collector configuration
    pub collector: Option<CollectorConfig>,
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
