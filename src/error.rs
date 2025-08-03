use log::error;
use miette::{Diagnostic, Report};
use thiserror::Error;

// Error builder
use bon::bon;

#[derive(Debug, Error, Diagnostic)]
pub enum MudrasError {
    /// Lib native error
    #[error(transparent)]
    #[diagnostic(transparent)]
    WrapError(#[from] WrapError),
    #[error(transparent)]
    #[diagnostic(code(io::error))]
    IoError(#[from] std::io::Error),
}

/// A config error with help higher origin
/// Can be recursively chained.
#[derive(Debug, Error, Diagnostic)]
#[error("{}", message)]
#[diagnostic(code(mudras::wrap::error))]
pub struct WrapError {
    pub message: String,
    #[diagnostic_source]
    pub origin: Report,
    #[help]
    pub help: String,
}

#[bon]
impl WrapError {
    #[builder]
    pub fn new(msg: &str, help: &str, origin: Report) -> Self {
        Self {
            message: msg.to_owned(),
            help: help.to_owned(),
            origin,
        }
    }
}
