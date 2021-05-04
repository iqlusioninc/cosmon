//! Sagan Abscissa Application

use crate::{commands::SaganCommand, config::SaganConfig};
use abscissa_core::{
    application,
    application::AppCell,
    config::{self, CfgCell},
    trace, Application, EntryPoint, FrameworkError, StandardPaths,
};
use abscissa_tokio::TokioComponent;

/// Application state
pub static APP: AppCell<SaganApplication> = AppCell::new();

/// Abscissa `Application` type
#[derive(Debug, Default)]
pub struct SaganApplication {
    /// Application's `sagan.toml` config settings
    config: CfgCell<SaganConfig>,

    /// Application state
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
    fn config(&self) -> config::Reader<SaganConfig> {
        self.config.read()
    }

    /// Borrow the application state immutably.
    fn state(&self) -> &application::State<Self> {
        &self.state
    }

    /// Register all components used by this application.
    fn register_components(&mut self, command: &Self::Cmd) -> Result<(), FrameworkError> {
        let mut components = self.framework_components(command)?;
        components.push(Box::new(TokioComponent::new()?));

        let mut component_registry = self.state.components_mut();
        component_registry.register(components)
    }

    /// Post-configuration lifecycle callback.
    fn after_config(&mut self, config: SaganConfig) -> Result<(), FrameworkError> {
        let mut component_registry = self.state.components_mut();
        component_registry.after_config(&config)?;
        self.config.set_once(config);
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
