//! Alerting configuration - Datadog
//!
use serde::{Deserialize, Serialize};

/// Alerting configuration for Datadog
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// API key
    pub dd_api_key: Option<String>,

    /// Alert threshold used for now in handle_poll_event() fn
    pub alert_threshold: Option<i64>,
}
