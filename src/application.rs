//! Sagan Abscissa Application

use crate::{
    commands::SaganCommand,
    config::{collector::CollectorConfig, SaganConfig},
    message,
    network::{self, Network},
    prelude::*,
};
use abscissa_core::{
    application, application::AppCell, config, trace, Application, EntryPoint, FrameworkError,
    StandardPaths,
};
use abscissa_tokio::TokioComponent;
use std::{collections::BTreeMap as Map, process};

/// Application state
pub static APPLICATION: AppCell<SaganApplication> = AppCell::new();

/// Obtain a read-only (multi-reader) lock on the application state.
///
/// Panics if the application state has not been initialized.
pub fn app_reader() -> application::lock::Reader<SaganApplication> {
    APPLICATION.read()
}

/// Obtain an exclusive mutable lock on the application state.
pub fn app_writer() -> application::lock::Writer<SaganApplication> {
    APPLICATION.write()
}

/// Obtain a read-only (multi-reader) lock on the application configuration.
///
/// Panics if the application configuration has not been loaded.
pub fn app_config() -> config::Reader<SaganApplication> {
    config::Reader::new(&APPLICATION)
}

/// Abscissa `Application` type
#[derive(Debug, Default)]
pub struct SaganApplication {
    /// Application's `sagan.toml` config settings
    config: Option<SaganConfig>,

    /// Application state
    state: application::State<Self>,

    /// Network state
    networks: Map<network::Id, Network>,
}

impl Application for SaganApplication {
    /// Entrypoint command for this application.
    type Cmd = EntryPoint<SaganCommand>;

    /// Application configuration.
    type Cfg = SaganConfig;

    /// Paths to resources within the application.
    type Paths = StandardPaths;

    /// Accessor for application configuration.
    fn config(&self) -> &SaganConfig {
        self.config.as_ref().expect("`sagan.toml` not loaded")
    }

    /// Borrow the application state immutably.
    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    /// Borrow the application state mutably.
    fn state_mut(&mut self) -> &mut application::State<Self> {
        &mut self.state
    }

    /// Register all components used by this application.
    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let mut components = self.framework_components(command)?;
        components.push(Box::new(TokioComponent::new()?));
        self.state.components.register(components)
    }

    /// Post-configuration lifecycle callback.
    fn after_config(&mut self, config: SaganConfig) -> Result<(), FrameworkError> {
        self.state.components.after_config(&config)?;

        if let Some(collector_config) = &config.collector {
            self.init_collector(collector_config);
        }

        self.config = Some(config);
        Ok(())
    }

    /// Get logging configuration from command-line options.
    fn tracing_config(&self, command: &EntryPoint<SaganCommand>) -> trace::Config {
        if command.verbose {
            trace::Config::verbose()
        } else {
            trace::Config::default()
        }
    }
}

impl SaganApplication {
    /// Initialize collector state
    fn init_collector(&mut self, collector_config: &CollectorConfig) {
        if let Some((address_to_team, client_id_to_team)) = collector_config.build_hashmaps() {
            for network in Network::from_config(
                &collector_config.networks,
                &collector_config.statsd,
                collector_config.metrics_prefix.clone(),
                Some(client_id_to_team),
                Some(address_to_team),
            ) {
                let network_id = network.id();
                info!("Registering network {}", network_id);
                if self.networks.insert(network_id.clone(), network).is_some() {
                    status_err!("duplicate networks in config: {}", &network_id);
                    process::exit(1);
                }
            }
        } else {
            for network in Network::from_config(
                &collector_config.networks,
                &collector_config.statsd,
                collector_config.metrics_prefix.clone(),
                None,
                None,
            ) {
                let network_id = network.id();
                info!("Registering network {}", network_id);
                if self.networks.insert(network_id.clone(), network).is_some() {
                    status_err!("duplicate networks in config: {}", &network_id);
                    process::exit(1);
                }
            }
        }
    }

    /// Borrow a network registered with this application
    pub fn network(&self, network_id: impl Into<network::Id>) -> Option<&Network> {
        self.networks.get(&network_id.into())
    }

    /// Handle an incoming status message from a monitor
    pub fn handle_message(&mut self, message: message::Envelope) {
        if let Some(network) = self.networks.get_mut(&message.network.into()) {
            network.handle_message(message);
        } else {
            warn!("got message for unregistered network: {}", &message.network);
        }
    }
}
