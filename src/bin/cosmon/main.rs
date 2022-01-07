//! Main entry point for Cosmon

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use cosmon::application::APP;

/// Boot Cosmon
fn main() {
    abscissa_core::boot(&APP);
}
