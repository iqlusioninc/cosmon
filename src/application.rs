//! cosmon Abscissa Application

use crate::{commands::EntryPoint, config::CosmonConfig};
use abscissa_core::{
    application,
    application::AppCell,
    config::{self, CfgCell},
    trace, Application, FrameworkError, StandardPaths,
};
use abscissa_tokio::TokioComponent;

/// Application state
pub static APP: AppCell<CosmonApplication> = AppCell::new();

/// Abscissa `Application` type
#[derive(Debug, Default)]
pub struct CosmonApplication {
    /// Application's `cosmon.toml` config settings
    config: CfgCell<CosmonConfig>,

    /// Application state
    state: application::State<Self>,
}

impl Application for CosmonApplication {
    /// Entrypoint command for this application.
    type Cmd = EntryPoint;

    /// Application configuration.
    type Cfg = CosmonConfig;

    /// Paths to resources within the application.
    type Paths = StandardPaths;

    /// Accessor for application configuration.
    fn config(&self) -> config::Reader<CosmonConfig> {
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
    fn after_config(&mut self, config: CosmonConfig) -> Result<(), FrameworkError> {
        let mut component_registry = self.state.components_mut();
        component_registry.after_config(&config)?;
        self.config.set_once(config);
        Ok(())
    }

    /// Get logging configuration from command-line options.
    fn tracing_config(&self, command: &EntryPoint) -> trace::Config {
        if command.verbose {
            trace::Config::verbose()
        } else {
            trace::Config::default()
        }
    }
}
