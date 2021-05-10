//! Response types

use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Response wrapper (i.e. message envelope)
// TODO(tarcieri): use an enum?
#[derive(Debug, Deserialize, Serialize)]
pub struct Wrapper<R> {
    /// Results of request (if successful)
    pub result: Option<R>,

    /// Error message if unsuccessful
    pub error: Option<String>,
}

impl<R> Wrapper<R>
where
    R: Serialize,
{
    /// Convert this wrapper into a result type
    pub fn from_result(result: Result<R, BoxError>) -> Self {
        match result {
            Ok(res) => Wrapper {
                result: Some(res),
                error: None,
            },
            Err(err) => Wrapper {
                result: None,
                error: Some(err.to_string()),
            },
        }
    }
}
