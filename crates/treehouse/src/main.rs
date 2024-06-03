use std::path::Path;

use clap::Parser;
use cli::{
    fix::{fix_all_cli, fix_file_cli},
    generate::regenerate_or_report_error,
    serve::serve,
    Command, Paths, ProgramArgs,
};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod cli;
mod config;
mod fun;
mod html;
mod paths;
mod state;
mod tree;

async fn fallible_main() -> anyhow::Result<()> {
    let args = ProgramArgs::parse();

    let paths = Paths {
        target_dir: Path::new("target/site"),
        template_target_dir: Path::new("target/site/static/html"),

        config_file: Path::new("treehouse.toml"),

        // NOTE: These are intentionally left unconfigurable from within treehouse.toml
        // because this is is one of those things that should be consistent between sites.
        static_dir: Path::new("static"),
        template_dir: Path::new("template"),
        content_dir: Path::new("content"),
    };

    match args.command {
        Command::Generate(_generate_args) => {
            info!("regenerating using directories: {paths:#?}");
            regenerate_or_report_error(&paths)?;
            warn!("`generate` is for debugging only and the files cannot be fully served using a static file server; use `treehouse serve` if you wish to start a treehouse server");
        }
        Command::Serve {
            generate: _,
            serve: serve_args,
        } => {
            let (config, treehouse) = regenerate_or_report_error(&paths)?;
            serve(config, treehouse, &paths, serve_args.port).await?;
        }

        Command::Fix(fix_args) => fix_file_cli(fix_args)?,
        Command::FixAll(fix_args) => fix_all_cli(fix_args, &paths)?,

        Command::Ulid => {
            let mut rng = rand::thread_rng();
            let ulid = ulid::Generator::new()
                .generate_with_source(&mut rng)
                .expect("failed to generate ulid");
            println!("{ulid}");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("could not set tracing subscriber");

    match fallible_main().await {
        Ok(_) => (),
        Err(error) => {
            error!("fatal: {error:?}");
            std::process::exit(1);
        }
    }

    Ok(())
}
