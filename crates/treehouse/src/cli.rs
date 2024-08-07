pub mod fix;
pub mod serve;
pub mod wc;

use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct ProgramArgs {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Regenerate the website.
    Generate(#[clap(flatten)] GenerateArgs),

    /// Populate missing metadata in blocks.
    Fix(#[clap(flatten)] FixArgs),

    /// Populate missing metadata in blocks across all files.
    ///
    /// By default only prints which files would be changed. To apply the changes, use `--apply`.
    FixAll(#[clap(flatten)] FixAllArgs),

    /// `generate` and start a treehouse server.
    ///
    /// The server uses the generated files and provides extra functionality on top, handling
    Serve {
        #[clap(flatten)]
        generate: GenerateArgs,

        #[clap(flatten)]
        serve: ServeArgs,
    },

    /// Count words in the treehouse's branches.
    Wc(#[clap(flatten)] WcArgs),

    /// Generates a new ulid and prints it to stdout.
    Ulid,
}

#[derive(Args)]
pub struct GenerateArgs {}

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

#[derive(Args)]
pub struct FixAllArgs {
    /// If you're happy with the suggested changes, specifying this will apply them to the file
    /// (overwrite it in place.)
    #[clap(long)]
    pub apply: bool,
}

#[derive(Args)]
pub struct ServeArgs {
    /// The port under which to serve the treehouse.
    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,
}

#[derive(Args)]
pub struct WcArgs {
    /// A list of paths to report the word counts of.
    /// If no paths are provided, the entire tree is word-counted.
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub struct Paths<'a> {
    pub target_dir: &'a Path,
    pub template_target_dir: &'a Path,

    pub static_dir: &'a Path,
    pub template_dir: &'a Path,
    pub content_dir: &'a Path,

    pub config_file: &'a Path,
}
