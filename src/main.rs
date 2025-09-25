mod cli;
mod config;
mod error;
mod input;

// Error handling
use error::MudrasError;
use miette::Result;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<(), MudrasError> {
    error::make_handler()?;
    Cli::run().await?;
    Ok(())
}
