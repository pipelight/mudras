// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Error
use crate::error::{MudrasError, WrapError};
use miette::{Result, Error};
use log::{error, trace};

#[derive(knuffel::Decode, Debug, PartialEq, Serialize)]
pub struct Config {
    #[knuffel(child, default)]
    pub binds: Binds,
}

#[derive(Debug,Default, PartialEq, Serialize)]
pub struct Binds(pub HashMap<String, Vec<String>>);

// #[derive(knuffel::Decode)]
// pub struct Action(Vec<String>);
//
// #[derive(PartialEq,Eq,Hash, knuffel::DecodeScalar )]
// pub struct Bind {
//     mode: Mode,
//     keys: Keys,
// }
//
// #[derive(Default,PartialEq, Eq, knuffel::DecodeScalar)]
// pub enum Mode {
//     Press,
//     #[default]
//     Release,
//     Repeat,
// }
//
// #[derive(PartialEq, Eq, knuffel::Decode)]
// pub struct Keys(String);



impl Config {
    /// Get config from crate directory
    fn debug_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("./config.kdl");
        return path;
    }
    /// Get config from FHS path.
    fn release_path() -> PathBuf {
        let mut path = PathBuf::new();
        path.push("/etc/mudras/config.kdl");
        return path;
    }
    /// Return configuration from default file path.
    pub fn get() -> Result<Self, MudrasError> {
        // let config = CONFIG.lock().unwrap().clone();
        // Ok(config)

        #[cfg(debug_assertions)]
        let path = Self::debug_path();

        #[cfg(not(debug_assertions))]
        let path = Self::release_path();

        let path = path.display().to_string();
        let config = match Self::from_file(&path) {
            Ok(v) => v,
            Err(e) => {
                let message = format!("Couldn't find a configuration file.",);
                let help = format!("Create a configuration file at /etc/virshle/confi.toml");
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
        Ok(config)
    }
    pub fn from_file(path: &str) -> Result<Self, MudrasError> {
        let string = fs::read_to_string(path)?;
        Self::from_kdl(&path,&string)
    }
    pub fn from_kdl(path: &str, string: &str) -> Result<Self, MudrasError> {
        let res: Config = knuffel::parse(path,string).unwrap();
        Ok(res)
    }
}
