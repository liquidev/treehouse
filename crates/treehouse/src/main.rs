use clap::Parser;
use cli::{
    regenerate::{self, regenerate_or_report_error},
    Command, ProgramArgs,
};

mod cli;
mod html;

async fn fallible_main() -> anyhow::Result<()> {
    let args = ProgramArgs::parse();

    match args.command {
        Command::Regenerate(regenerate_args) => {
            regenerate_or_report_error();

            if regenerate_args.serve {
                regenerate::web_server().await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match fallible_main().await {
        Ok(_) => (),
        Err(error) => eprintln!("fatal: {error:?}"),
    }

    Ok(())
}
