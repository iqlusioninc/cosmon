//! `start` subcommand

use crate::{monitor::Monitor, prelude::*};
use abscissa_core::{Command, Options, Runnable};
use std::process;

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCommand {}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        // TODO(tarcieri): support collector in addition to monitor
        if let Some(mut monitor) = self.init_monitor() {
            monitor.run();
        }
    }
}

impl StartCommand {
    /// Initialize the monitor (if configured)
    fn init_monitor(&self) -> Option<Monitor> {
        let app = app_reader();

        app.config().agent.as_ref().map(|agent_config| {
            Monitor::new(&agent_config.node_home, app.tendermint_config()).unwrap_or_else(|e| {
                status_err!("couldn't initialize monitor: {}", e);
                process::exit(1);
            })
        })
    }
}
