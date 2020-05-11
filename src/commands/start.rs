//! `start` subcommand

use crate::{
    application::APPLICATION,
    collector::HttpServer,
    event_monitor::{EventMonitor, EventReporter},
    monitor::Monitor,
    prelude::*,
};
use abscissa_core::{Command, Options, Runnable};
use std::process;
use tendermint::net;
use tokio::sync::mpsc::channel;

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCommand {}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APPLICATION, async {
            if let Some(listen_addr) = self.init_collector() {
                let c_handle = tokio::spawn(async move {
                    let collector = HttpServer::new(&listen_addr).unwrap_or_else(|e| {
                        status_err!("couldn't initialize HTTP collector: {}", e);
                        process::exit(1);
                    });

                    collector.run();
                });
                c_handle.await.unwrap();
            }

            if let Some((mut monitor, mut event_monitor, mut event_reporter)) =
                self.init_monitor().await
            {
                let m_handle = tokio::spawn(async move { monitor.run().await });

                let e_handle = tokio::spawn(async move { event_monitor.run().await });

                let r_handle = tokio::spawn(async move { event_reporter.run().await });
                let (r0, r1, r2) = futures::join!(m_handle, e_handle, r_handle);

                r0.unwrap();
                r1.unwrap();
                r2.unwrap();
            }
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
    async fn init_monitor(&self) -> Option<(Monitor, EventMonitor, EventReporter)> {
        let app = app_reader();

        if let Some(agent_config) = &app.config().agent {
            let monitor = Monitor::new(agent_config).await.unwrap_or_else(|e| {
                status_err!("couldn't initialize monitor: {}", e);
                process::exit(1);
            });
            let (chain_id, node_id) = monitor.id();
            let (tx, rx) = channel(100);
            let event_monitor = EventMonitor::new(agent_config, tx)
                .await
                .unwrap_or_else(|e| {
                    status_err!("couldn't initialize event listener: {}", e);
                    process::exit(1);
                });
            let event_reporter = EventReporter::new(agent_config, rx, node_id, chain_id);
            Some((monitor, event_monitor, event_reporter))
        } else {
            trace!("Agent config not found");
            None
        }
    }
}
