use crate::cli::{Cli, Command};
use crate::error::RegistryError;
use crate::registry::Registry;

use super::build::build;
use super::check::check;
use super::publish::publish;

pub fn run(cli: &Cli) -> Result<(), RegistryError> {
    let registry = Registry::load(&cli.root)?;
    let extensions = check(&cli.root, &registry)?;

    match cli.command {
        Command::Check => Ok(()),
        Command::Build => build(&extensions, &cli.out),
        Command::Publish => publish(&cli.root, &extensions, &cli.out),
    }
}
