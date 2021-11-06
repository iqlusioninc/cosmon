//! cosmon acceptance tests

/*#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use abscissa_core::testing::prelude::*;
use cosmon::config::CosmonConfig;
use once_cell::sync::Lazy;

pub static RUNNER: Lazy<CmdRunner> = Lazy::new(CmdRunner::default);*/

// TODO https://github.com/iqlusioninc/abscissa/issues/584
/*#[test]
fn start() {
    let mut runner = RUNNER.clone();
    let cmd = runner
        .config(&CosmonConfig::default())
        .arg("start")
        .capture_stdout()
        .run();

    cmd.wait().unwrap().expect_success();
}*/
