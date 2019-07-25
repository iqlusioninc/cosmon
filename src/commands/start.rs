//! `start` subcommand

use crate::prelude::*;
use abscissa_core::{Command, Options, Runnable};

/// `start` subcommand
#[derive(Command, Debug, Options)]
pub struct StartCommand {
    /// To whom are we saying hello?
    #[options(free)]
    recipient: Vec<String>,
}

impl Runnable for StartCommand {
    /// Start the application.
    fn run(&self) {
        let app = app_reader();

        if app.config().is_agent() {
            println!("Tendermint config: {:?}", app.tendermint_config());
        }
    }
}
