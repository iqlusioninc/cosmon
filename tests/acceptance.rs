//! Sagan acceptance tests

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use abscissa_core::testing::prelude::*;
use once_cell::sync::Lazy;
use sagan::config::SaganConfig;

pub static RUNNER: Lazy<CmdRunner> = Lazy::new(CmdRunner::default);

#[test]
fn start() {
    let mut runner = RUNNER.clone();
    let cmd = runner
        .config(&SaganConfig::default())
        .arg("start")
        .capture_stdout()
        .run();

    cmd.wait().unwrap().expect_success();
}
