mod cli;
mod commands;
mod component;
mod error;
mod index;
mod manifest;
mod process;
mod registry;
mod rules;
mod signer;
mod toml_file;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match commands::run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("registry: {error}");
            ExitCode::FAILURE
        }
    }
}
