use std::path::Path;

use clap::Parser;
use cli::{
    fix::{fix_all_cli, fix_file_cli},
    generate::{self, regenerate_or_report_error},
    Command, Paths, ProgramArgs,
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

    let paths = Paths {
        target_dir: Path::new("target/site"),
        config_file: Path::new("treehouse.toml"),

        // NOTE: These are intentionally left unconfigurable from within treehouse.toml
        // because this is is one of those things that should be consistent between sites.
        static_dir: Path::new("static"),
        template_dir: Path::new("template"),
        content_dir: Path::new("content"),
    };

    match args.command {
        Command::Generate(regenerate_args) => {
            info!("regenerating using directories: {paths:#?}");

            regenerate_or_report_error(&paths);

            if let Some(port) = regenerate_args.serve {
                generate::web_server(port).await?;
            }
        }

        Command::Fix(fix_args) => fix_file_cli(fix_args)?,
        Command::FixAll(fix_args) => fix_all_cli(fix_args, &paths)?,
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
