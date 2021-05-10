//! `start` subcommand

use crate::{
    application::APP,
    collector::{self, Collector},
    monitor::Monitor,
    prelude::*,
};
use abscissa_core::{Command, Options, Runnable};
use futures::future;
use std::process;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCommand {}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            let mut tasks = vec![];

            if let Some(poller) = self.init_collector_poller().await {
                tasks.push(poller);
            }

            if let Some(router) = self.init_collector_router().await {
                tasks.push(router);
            }

            if let Some(monitor) = self.init_monitor().await {
                tasks.push(monitor);
            }

            future::join_all(tasks).await;
        })
        .expect("Tokio runtime crashed");
    }
}

#[allow(clippy::manual_map)] // TODO(tarcieri): use async closures when stable
impl StartCommand {
    /// Initialize collector poller (if configured/needed)
    async fn init_collector_poller(&self) -> Option<JoinHandle<()>> {
        if let Some(config) = APP.config().collector.clone() {
            Some(tokio::spawn(async move {
                let poller = collector::Poller::new(&config).unwrap_or_else(|e| {
                    status_err!("couldn't initialize collector poller: {}", e);
                    process::exit(1);
                });

                poller.run().await;
            }))
        } else {
            None
        }
    }

    /// Initialize collector (if configured)
    async fn init_collector_router(&self) -> Option<JoinHandle<()>> {
        if let Some(config) = APP.config().collector.clone() {
            Some(tokio::spawn(async move {
                let collector = ServiceBuilder::new().buffer(20).service(
                    Collector::new(&config).unwrap_or_else(|e| {
                        status_err!("couldn't initialize collector service: {}", e);
                        process::exit(1);
                    }),
                );

                let router = collector::Router::new(&config);
                router.run(collector).await;
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
