pub mod diagnostics;
pub mod fix;
mod parse;
pub mod regenerate;

use std::path::PathBuf;

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

    /// Populate missing metadata in blocks.
    Fix(#[clap(flatten)] FixArgs),
}

#[derive(Args)]
pub struct RegenerateArgs {
    /// Start a web server serving the static files. Useful with `cargo watch`.
    #[clap(short, long)]
    pub serve: bool,
}

#[derive(Args)]
pub struct FixArgs {
    /// Which file to fix. The fixed file will be printed into stdout so that you have a chance to
    /// see the changes.
    pub file: PathBuf,

    /// If you're happy with the suggested changes, specifying this will apply them to the file
    /// (overwrite it in place.)
    #[clap(long)]
    pub apply: bool,

    /// Write the previous version back to the specified path.
    #[clap(long)]
    pub backup: Option<PathBuf>,
}
