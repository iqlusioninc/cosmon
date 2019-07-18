//! `start` subcommand

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
        println!(
            "The cosmos is full beyond measure of elegant truths; \
             of exquisite interrelationships; of the awesome machinery of nature."
        );
    }
}
