//! Error types

use abscissa_core::error::{BoxError, Context};
use std::ops::Deref;
use std::{fmt, io};
use thiserror::Error;

/// Error type
#[derive(Debug)]
pub struct Error(Box<Context<ErrorKind>>);

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
pub enum ErrorKind {
    /// Error in configuration file
    #[error("config error")]
    ConfigError,

    /// Input/output error
    #[error("I/O error")]
    IoError,

    /// Error reporting events to the collector
    #[error("error reporting to collector")]
    ReportError,

    /// Error performing an RPC to the Tendermint node
    #[error("RPC request error")]
    RpcError,
}

impl ErrorKind {
    /// Create an error context from this error
    pub fn context(self, source: impl Into<BoxError>) -> Context<ErrorKind> {
        Context::new(self, Some(source.into()))
    }
}

impl Deref for Error {
    type Target = Context<ErrorKind>;

    fn deref(&self) -> &Context<ErrorKind> {
        &self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(context: Context<ErrorKind>) -> Self {
        Error(Box::new(context))
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        ErrorKind::IoError.context(other).into()
    }
}

impl From<tendermint::Error> for Error {
    fn from(other: tendermint::Error) -> Error {
        // TODO(tarcieri): better error conversions
        format_err!(ErrorKind::ConfigError, "{}", other).into()
    }
}

impl From<tendermint_rpc::Error> for Error {
    fn from(other: tendermint_rpc::Error) -> Error {
        // TODO(tarcieri): better error conversions
        format_err!(ErrorKind::RpcError, "{}", other).into()
    }
}
