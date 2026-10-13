use std::path::PathBuf;

use clap::Parser;

use super::Command;

#[derive(Debug, Parser)]
#[command(
    name = "registry",
    about = "Check, build, sign and index the Synt extensions"
)]
pub struct Cli {
    #[arg(
        long,
        default_value = ".",
        value_name = "PATH",
        help = "Folder that holds extensions.toml"
    )]
    pub root: PathBuf,

    #[arg(
        long,
        default_value = "dist",
        value_name = "PATH",
        help = "Folder for the built components, their signatures and index.json"
    )]
    pub out: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}
