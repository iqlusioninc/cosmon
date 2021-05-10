//! Network state

use super::tendermint;
use serde::Serialize;

/// Network state
#[derive(Debug, Serialize)]
pub enum State {
    /// Tendermint network state
    #[serde(rename = "tendermint")]
    Tendermint(tendermint::State),
}

impl From<tendermint::State> for State {
    fn from(state: tendermint::State) -> State {
        Self::Tendermint(state)
    }
}
