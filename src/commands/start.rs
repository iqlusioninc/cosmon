//! `start` subcommand

use crate::{application::APPLICATION, collector::HttpServer, monitor::Monitor, prelude::*};
use abscissa_core::{Command, Options, Runnable};
use std::process;
use tendermint::net;

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCommand {}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APPLICATION, async {
            self.init_collector().map(|listen_addr| {
                tokio::spawn(async move {
                    let collector = HttpServer::new(&listen_addr).unwrap_or_else(|e| {
                        status_err!("couldn't initialize HTTP collector: {}", e);
                        process::exit(1);
                    });

                    collector.run();
                })
            });

            self.init_monitor().await.map(|mut monitor| {
                tokio::spawn(async move {
                    monitor.run().await;
                })
            });
        })
        .unwrap();
    }
}

impl StartCommand {
    /// Initialize the collector (if configured)
    fn init_collector(&self) -> Option<net::Address> {
        let app = app_reader();

        app.config()
            .collector
            .as_ref()
            .map(|collector_config| collector_config.listen_addr.clone())
    }

    /// Initialize the monitor (if configured)
    async fn init_monitor(&self) -> Option<Monitor> {
        let app = app_reader();

        if let Some(agent_config) = &app.config().agent {
            Some(Monitor::new(agent_config).await.unwrap_or_else(|e| {
                status_err!("couldn't initialize monitor: {}", e);
                process::exit(1);
            }))
        } else {
            None
        }
    }
}
