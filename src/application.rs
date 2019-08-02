//! Sagan Abscissa Application

use crate::{commands::SaganCommand, config::SaganConfig};
use abscissa_core::{
    application, config, logging, Application, EntryPoint, FrameworkError, StandardPaths,
};
use lazy_static::lazy_static;

lazy_static! {
    /// Application state
    pub static ref APPLICATION: application::Lock<SaganApplication> = application::Lock::default();
}

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

    /// Application state.
    state: application::State<Self>,
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
        let components = self.framework_components(command)?;
        self.state.components.register(components)
    }

    /// Post-configuration lifecycle callback.
    fn after_config(&mut self, config: SaganConfig) -> Result<(), FrameworkError> {
        self.state.components.after_config(&config)?;
        self.config = Some(config);
        Ok(())
    }

    /// Get logging configuration from command-line options.
    fn logging_config(&self, command: &EntryPoint<SaganCommand>) -> logging::Config {
        if command.verbose {
            logging::Config::verbose()
        } else {
            logging::Config::default()
        }
    }
}
