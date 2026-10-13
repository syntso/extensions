use clap::Subcommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Command {
    #[command(about = "Check extensions.toml and every extension.toml it lists")]
    Check,

    #[command(about = "Check, then build every extension into a component and inspect it")]
    Build,

    #[command(about = "Sign the built components and write index.json")]
    Publish,
}
