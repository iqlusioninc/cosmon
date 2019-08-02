//! Database directory monitor

use super::message::Message;
use crate::error::Error;
use std::path::PathBuf;

/// Database directory monitor: monitors the `data/` directory
// TODO(tarcieri): actually implement this properly
#[derive(Clone, Debug)]
pub struct Data {
    /// Path to the database
    #[allow(dead_code)]
    path: PathBuf,
}

impl Data {
    /// Create a new network information monitor
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Update internal state, returning any changes
    pub fn update(&mut self, _force: bool) -> Result<Vec<Message>, Error> {
        let output = vec![];
        Ok(output)
    }
}
