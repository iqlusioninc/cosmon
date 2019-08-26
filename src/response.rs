//! Response types

use serde::{Deserialize, Serialize};

/// Response wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
pub struct Wrapper<R> {
    /// Results of request (if successful)
    pub result: Option<R>,

    /// Error message if unsuccessful
    pub error: Option<Error>,
}

impl<R> Wrapper<R>
where
    R: Serialize,
{
    /// Convert this wrapper into a result type
    pub fn from_result(result: Result<R, Error>) -> Self {
        match result {
            Ok(res) => Wrapper {
                result: Some(res),
                error: None,
            },
            Err(err) => Wrapper {
                result: None,
                error: Some(err),
            },
        }
    }
}

/// Error type
#[derive(Debug, Deserialize, Serialize)]
pub struct Error {}
