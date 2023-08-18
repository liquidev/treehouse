pub mod regenerate;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct ProgramArgs {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Regenerate the website.
    Regenerate(#[clap(flatten)] RegenerateArgs),
}

#[derive(Args)]
pub struct RegenerateArgs {
    /// Start a web server serving the static files. Useful with `cargo watch`.
    #[clap(short, long)]
    pub serve: bool,
}
