//! Sagan configuration file

use abscissa_core::Config;
use serde::{Deserialize, Serialize};

/// Sagan config
#[derive(Clone, Config, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SaganConfig {}
