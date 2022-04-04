//! Alert configuration

pub mod datadog;

use serde::{Deserialize, Serialize};

/// Types of alerting platforms
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Datadog config
    #[serde(default)]
    pub datadog: Option<datadog::Config>,
}
