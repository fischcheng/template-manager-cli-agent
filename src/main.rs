mod agent;
mod app;
mod cli;
mod doctor;
mod error;
mod fs;
mod manifest;
mod scaffold;
mod spec_kit;
mod update;

use std::process::ExitCode;

use clap::Parser;

use crate::app::run;
use crate::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}
