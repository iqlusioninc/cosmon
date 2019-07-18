//! Sagan subcommands

mod start;
mod version;

use self::{start::StartCommand, version::VersionCommand};
use crate::config::SaganConfig;
use abscissa_core::{Command, Configurable, Help, Options, Runnable};
use std::path::PathBuf;

/// Configuration filename
pub const CONFIG_FILE: &str = "sagan.toml";

/// Subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum SaganCommand {
    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `start` subcommand
    #[options(help = "start the application")]
    Start(StartCommand),

    /// The `version` subcommand
    #[options(help = "display version information")]
    Version(VersionCommand),
}

impl Configurable<SaganConfig> for SaganCommand {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        Some(PathBuf::from(CONFIG_FILE))
    }
}
