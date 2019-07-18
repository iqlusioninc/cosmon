//! Sagan acceptance tests

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use abscissa_core::testing::prelude::*;
use lazy_static::lazy_static;
use sagan::config::SaganConfig;

lazy_static! {
    pub static ref RUNNER: CmdRunner = CmdRunner::default();
}

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
