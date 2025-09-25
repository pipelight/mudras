mod utils;

use crate::config::Config;
use crate::input;

use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use clap_verbosity_flag::{InfoLevel, Verbosity};

// Error Handling
use crate::error::MudrasError;
use miette::Result;
use tracing::info;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
    #[command(flatten)]
    pub verbose: Verbosity,
}

#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Commands {
    #[command()]
    Run,
}

impl Cli {
    pub async fn run() -> Result<(), MudrasError> {
        let cli = Cli::parse();
        Self::switch(cli).await?;
        Ok(())
    }

    pub async fn switch(cli: Cli) -> Result<(), MudrasError> {
        utils::set_tracer(&cli)?;
        utils::set_logger(&cli)?;

        match cli.commands {
            Commands::Run => {
                info!("Running node.");
                let config = Config::get()?;
                input::listen_keyboard(&config).await?;
                Ok(())
            }
        }
    }
}
