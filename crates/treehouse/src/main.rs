use std::path::Path;

use clap::Parser;
use cli::{
    fix::fix_file_cli,
    regenerate::{self, regenerate_or_report_error, Paths},
    Command, ProgramArgs,
};
use log::{error, info};

mod cli;
mod config;
mod html;
mod paths;
mod state;
mod tree;

async fn fallible_main() -> anyhow::Result<()> {
    let args = ProgramArgs::parse();

    match args.command {
        Command::Regenerate(regenerate_args) => {
            let dirs = Paths {
                target_dir: Path::new("target/site"),
                config_file: Path::new("treehouse.toml"),

                // NOTE: These are intentionally left unconfigurable from within treehouse.toml
                // because this is is one of those things that should be consistent between sites.
                static_dir: Path::new("static"),
                template_dir: Path::new("template"),
                content_dir: Path::new("content"),
            };
            info!("regenerating using directories: {dirs:#?}");

            regenerate_or_report_error(&dirs);

            if regenerate_args.serve {
                regenerate::web_server().await?;
            }
        }

        Command::Fix(fix_args) => fix_file_cli(fix_args)?,
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_module("treehouse", log::LevelFilter::Debug)
        .init();

    match fallible_main().await {
        Ok(_) => (),
        Err(error) => error!("fatal: {error:?}"),
    }

    Ok(())
}
