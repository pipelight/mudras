mod config;
mod error;
mod input;

use miette::{Error, IntoDiagnostic, MietteHandlerOpts, Result, RgbColors};

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    make_handler()?;
    let config = Config::get().into_diagnostic()?;
    input::listen_keyboard().await?;
    println!("{:#?}", config);
    Ok(())
}
/// The make handler functions is executed right after the main function
/// to set up a verbose and colorful error/panic handler.
pub fn make_handler() -> Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .rgb_colors(RgbColors::Never)
                .color(true)
                .unicode(true)
                .terminal_links(true)
                .context_lines(3)
                .with_cause_chain()
                .build(),
        )
    }))?;
    miette::set_panic_hook();
    Ok(())
}
