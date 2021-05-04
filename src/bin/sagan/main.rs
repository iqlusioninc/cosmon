//! Main entry point for Sagan

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use sagan::application::APP;

/// Boot Sagan
fn main() {
    abscissa_core::boot(&APP);
}
