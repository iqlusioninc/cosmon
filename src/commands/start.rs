//! `start` subcommand

use crate::{collector, monitor::Monitor, prelude::*};
use abscissa_core::{Command, Options, Runnable};
use std::{process, thread};
use tendermint::net;

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCommand {}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        let collector_thread = self.init_collector().map(|listen_addr| {
            thread::spawn(move || {
                collector::run(&listen_addr);
            })
        });

        let monitor_thread = self.init_monitor().map(|mut monitor| {
            thread::spawn(move || {
                monitor.run();
            })
        });

        if let Some(child) = collector_thread {
            child.join().unwrap();
        }

        if let Some(child) = monitor_thread {
            child.join().unwrap();
        }
    }
}

impl StartCommand {
    /// Initialize the collector (if configured)
    fn init_collector(&self) -> Option<net::Address> {
        let app = app_reader();

        app.config().collector.as_ref().map(|collector_config| {
            collector_config.listen_addr.clone()
        })
    }

    /// Initialize the monitor (if configured)
    fn init_monitor(&self) -> Option<Monitor> {
        let app = app_reader();

        app.config().agent.as_ref().map(|agent_config| {
            Monitor::new(agent_config).unwrap_or_else(|e| {
                status_err!("couldn't initialize monitor: {}", e);
                process::exit(1);
            })
        })
    }


}
