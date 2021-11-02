//! `version` subcommand

#![allow(clippy::never_loop)]

use super::CosmonCommand;
use abscissa_core::{Command, Options, Runnable};

/// `version` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct VersionCommand {}

impl Runnable for VersionCommand {
    /// Print version message
    fn run(&self) {
        println!("{} {}", CosmonCommand::name(), CosmonCommand::version());
    }
}
