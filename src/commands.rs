//! cosmon subcommands

mod start;

use self::start::StartCommand;
use crate::config::CosmonConfig;
use abscissa_core::{Command, Configurable, Runnable};
use clap::Parser;
use std::path::PathBuf;

/// Configuration filename
pub const CONFIG_FILE: &str = "cosmon.toml";

/// Subcommands
#[derive(Command, Debug, Parser, Runnable)]
pub enum CosmonCommand {
    /// The `start` subcommand
    #[clap()]
    Start(StartCommand),
}

impl Configurable<CosmonConfig> for EntryPoint {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        Some(PathBuf::from(CONFIG_FILE))
    }
}

/// Entry point for the application.
#[derive(Command, Debug, Parser)]
#[clap(author, about, version)]
pub struct EntryPoint {
    #[clap(subcommand)]
    /// CosmonCommand
    cmd: CosmonCommand,

    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,

    /// Use the specified config file
    #[clap(short, long)]
    pub config: Option<String>,
}

impl Runnable for EntryPoint {
    fn run(&self) {
        self.cmd.run()
    }
}
