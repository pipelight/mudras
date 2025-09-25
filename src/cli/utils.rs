use super::Cli;

// Logger
use env_logger::Builder;

/// Tracing
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

// Error handling
use crate::error::MudrasError;
use miette::Result;

/// Build tracing
pub fn set_tracer(cli: &Cli) -> Result<(), MudrasError> {
    // Set verbosity
    let verbosity: Level = cli.verbose.tracing_level().unwrap();
    let filter = format!(
        "{},{}",
        verbosity.to_string().to_lowercase(),
        "udev=error,evdev=error"
    );
    let builder = FmtSubscriber::builder()
        .with_max_level(verbosity)
        .with_env_filter(EnvFilter::try_new(filter).unwrap());

    #[cfg(debug_assertions)]
    let builder = builder.pretty();
    #[cfg(not(debug_assertions))]
    let builder = builder.compact();

    let subscriber = builder.finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    Ok(())
}
/// Build logger
pub fn set_logger(cli: &Cli) -> Result<(), MudrasError> {
    // Set verbosity
    let verbosity: log::LevelFilter = cli.verbose.log_level_filter();
    // Disable sql logs
    let filter = format!(
        "{},{}",
        verbosity.to_string().to_lowercase(),
        "mio=error,sqlx=error,russh=error,users=warn"
    );
    std::env::set_var("CROTUI_LOG", filter);
    Builder::from_env("CROTUI_LOG").init();

    Ok(())
}
