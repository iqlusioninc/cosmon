//! Application-local prelude: conveniently import types/functions/macros
//! which are generally useful and should be available everywhere.

/// Abscissa core prelude.
pub use abscissa_core::prelude::*;

/// Other Abscissa imports.
pub use abscissa_core::error::BoxError;

/// Application-level types.
pub use crate::{
    application::APP,
    config::CosmonConfig,
    error::{Error, ErrorKind},
};

/// Map type.
pub use std::collections::BTreeMap as Map;
