//! Application-local prelude: conveniently import types/functions/macros
//! which are generally useful and should be available everywhere.

/// Abscissa core prelude.
pub use abscissa_core::prelude::*;

/// Application-level types.
pub use crate::{
    application::APP,
    config::SaganConfig,
    error::{Error, ErrorKind},
};

/// Map type.
pub use std::collections::BTreeMap as Map;
