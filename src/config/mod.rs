mod kdl;
pub mod utils;

pub use kdl::*;

// Config
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Error
use crate::error::{MudrasError, WrapError};
use miette::{Error, Result};
use tracing::{error, trace};

impl Config {
    /// Get config from crate directory
    fn debug_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("./config.kdl");
        return path;
    }
    /// Get config from FHS path.
    fn release_path() -> Result<PathBuf, MudrasError> {
        let mut path = PathBuf::new();
        path.push(utils::shellexpand("/home/anon/.config/mudras/config.kdl")?);
        Ok(path)
    }
    /// Return configuration from default file path.
    pub fn get() -> Result<Self, MudrasError> {
        #[cfg(debug_assertions)]
        let path = Self::debug_path();

        #[cfg(not(debug_assertions))]
        let path = Self::release_path()?;
        let path = path.display().to_string();

        let config = match Self::from_file(&path) {
            Ok(v) => v,
            Err(e) => {
                let message = format!("Couldn't find a configuration file.",);
                let help = format!("Create a configuration file at ~/.config/mudras.kdl");
                let err = WrapError::builder()
                    .msg(&message)
                    .help(&help)
                    .origin(Error::from_err(e))
                    .build();

                error!("{}", err);
                return Err(err.into());
            }
        };
        trace!("Found config file.");
        trace!("{:#?}", config);
        Ok(config)
    }
    pub fn from_file(path: &str) -> Result<Self, MudrasError> {
        let string = fs::read_to_string(path)?;
        Self::from_kdl(&path, &string)
    }
    pub fn from_kdl(path: &str, string: &str) -> Result<Self, MudrasError> {
        let res: Config = knus::parse(path, string)
            .map_err(miette::Report::new)
            .unwrap();
        Ok(res)
    }
}

// Getters
impl Config {
    /// Get submaps list.
    pub fn get_submaps(&self) -> Result<HashMap<String, Vec<Bind>>, MudrasError> {
        let mut list = HashMap::new();

        // Create a root submap from root binds.
        let main_binds: Vec<Bind> = self
            .items
            .clone()
            .into_iter()
            .filter_map(|e| match e {
                Item::Bind(bind) => Some(bind),
                _ => None,
            })
            .collect();
        list.insert("main".to_owned(), main_binds);

        // Add other submaps to list.
        for e in self.items.clone() {
            match e {
                Item::Submap(submap) => {
                    list.insert(submap.name, submap.binds);
                }
                _ => {}
            }
        }

        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    #[test]
    fn parse_config_file() -> Result<()> {
        let config = Config::get()?;
        println!("{:#?}", config);
        Ok(())
    }
}
