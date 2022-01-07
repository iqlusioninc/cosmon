//! `start` subcommand

use crate::{
    application::APP,
    collector::{self, Collector},
    config,
    monitor::Monitor,
    prelude::*,
};
use abscissa_core::{Command, Runnable};
use clap::Parser;
use futures::future;
use std::process;
use tokio::task::JoinHandle;
use tower::{Service, ServiceBuilder};

/// `start` subcommand
#[derive(Command, Debug, Parser)]
pub struct StartCommand {}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            let mut tasks = self.init_collector().await;

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
    /// Initialize the collector
    async fn init_collector(&self) -> Vec<JoinHandle<()>> {
        let mut tasks = vec![];

        if let Some(config) = APP.config().collector.clone() {
            let collector =
                ServiceBuilder::new()
                    .buffer(20)
                    .service(Collector::new(&config).unwrap_or_else(|e| {
                        status_err!("couldn't initialize collector service: {}", e);
                        process::exit(1);
                    }));

            tasks.push(
                self.init_collector_poller(config.clone(), collector.clone())
                    .await,
            );

            tasks.push(
                self.init_collector_router(config.clone(), collector.clone())
                    .await,
            );
        }

        tasks
    }

    /// Initialize collector poller (if configured/needed)
    async fn init_collector_poller<S>(
        &self,
        config: config::collector::Config,
        collector: S,
    ) -> JoinHandle<()>
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Sync
            + Clone
            + 'static,
        S::Future: Send,
    {
        tokio::spawn(async move {
            let poller = collector::Poller::new(&config).unwrap_or_else(|e| {
                status_err!("couldn't initialize collector poller: {}", e);
                process::exit(1);
            });

            poller.run(collector).await;
        })
    }

    /// Initialize collector (if configured)
    async fn init_collector_router<S>(
        &self,
        config: config::collector::Config,
        collector: S,
    ) -> JoinHandle<()>
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Sync
            + Clone
            + 'static,
        S::Future: Send,
    {
        tokio::spawn(async move {
            let router = collector::Router::new(&config);
            router.run(collector).await;
        })
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
