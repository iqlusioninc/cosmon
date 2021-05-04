//! `start` subcommand

use crate::{application::APP, collector, monitor::Monitor, prelude::*};
use abscissa_core::{Command, Options, Runnable};
use futures::future;
use std::process;
use tokio::task::JoinHandle;

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCommand {}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            let mut tasks = vec![];

            if let Some(collector) = self.init_collector().await {
                tasks.push(collector);
            }

            if let Some(monitor) = self.init_monitor().await {
                tasks.push(monitor);
            }

            future::join_all(tasks).await;
        })
        .expect("Tokio runtime crashed");
    }
}

impl StartCommand {
    /// Initialize collector (if configured)
    async fn init_collector(&self) -> Option<JoinHandle<()>> {
        if let Some(config) = APP.config().collector.clone() {
            Some(tokio::spawn(async move {
                let collector = collector::Router::new(&config).unwrap_or_else(|e| {
                    status_err!("couldn't initialize HTTP collector: {}", e);
                    process::exit(1);
                });

                collector.run().await;
            }))
        } else {
            None
        }
    }

    /// Initialize monitor (if configured)
    async fn init_monitor(&self) -> Option<JoinHandle<()>> {
        if let Some(config) = APP.config().agent.clone() {
            let mut monitor = Monitor::new(&config).await.unwrap_or_else(|e| {
                status_err!("couldn't initialize monitor: {}", e);
                process::exit(1);
            });

            Some(tokio::spawn(async move {
                monitor.run().await;
            }))
        } else {
            None
        }
    }
}
