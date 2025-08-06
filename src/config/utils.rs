use std::path::Path;

// Error
use crate::error::{LibError, MudrasError};
use log::{error, trace};
use miette::{Error, IntoDiagnostic, Result};

/// Expand tild "~" in file path.
pub fn shellexpand(relpath: &str) -> Result<String, MudrasError> {
    let source: String = match relpath.starts_with("~") {
        false => relpath.to_owned(),
        true => relpath.replace("~", dirs::home_dir().unwrap().to_str().unwrap()),
    };

    let path = Path::new(&source);
    if path.exists() {
        Ok(source)
    } else {
        let message = format!("Couldn't find file {:#?} expended to {:#?}.", relpath, path);
        error!("{:#?}", message);
        let err = LibError::builder()
            .msg(&message)
            .help("Are you sure the file exist?")
            .build();
        return Err(err.into());
    }
}
