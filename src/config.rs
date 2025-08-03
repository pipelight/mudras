// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Parse
use knus::errors::DecodeError;
use knus::Decode;

// Error
use crate::error::{MudrasError, WrapError};
use log::{error, trace};
use miette::{Error, IntoDiagnostic, Result};

#[derive(Debug, PartialEq, Serialize)]
pub struct Config(pub Vec<Bind>);
impl<S> knus::DecodeChildren<S> for Config
where
    S: knus::traits::ErrorSpan,
{
    fn decode_children(
        nodes: &[knus::ast::SpannedNode<S>],
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let mut binds = Vec::new();
        for node in nodes {
            let bind = Bind::decode_node(node, ctx)?;
            binds.push(bind);
        }
        Ok(Self(binds))
    }
}

#[derive(Default, Debug, PartialEq, Serialize)]
pub struct Bind {
    pub key: String,
    #[serde(flatten)]
    pub actions: Action,
}

#[derive(knus::Decode, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub struct Action {
    pub press: Option<Vec<String>>,
    pub release: Option<Vec<String>>,
    pub repeat: Option<Vec<String>>,
}

impl<S> knus::Decode<S> for Bind
where
    S: knus::traits::ErrorSpan,
{
    fn decode_node(
        node: &knus::ast::SpannedNode<S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        println!("{:#?}", node);
        println!("{:#?}", ctx);

        // if let Some(type_name) = &node.type_name {
        //     ctx.emit_error(DecodeError::unexpected(
        //         type_name,
        //         "type name",
        //         "no type name expected for this node",
        //     ));
        // }
        //
        // for val in node.arguments.iter() {
        //     ctx.emit_error(DecodeError::unexpected(
        //         &val.literal,
        //         "argument",
        //         "no arguments expected for this node",
        //     ));
        // }

        let press = None;
        let release = None;
        let repeat = None;

        let key = node.node_name.parse::<String>().unwrap();
        for child in node.children() {
            let name: String = child.node_name.to_string();
            match &*name {
                "@press" => {
                    let actions: Vec<String> = child.decode_node();
                    let press = Some(actions);
                }
                "@release" => {
                    release = child;
                }
                "@repeat" => {
                    repeat = child;
                }
            }
        }
        // for child in node.children() {
        //     let name: String = child.node_name.to_string();
        //     match &*name {
        //         "@press" => {
        //             press = child.;
        //         }
        //         "@release" => {
        //             release = child;
        //         }
        //         "@repeat" => {
        //             repeat = child;
        //         }
        //     }
        // }

        // let mode = parse_arg_node("mode", node, ctx)?;

        Ok(Self { key, actions })
    }
}

// #[derive(PartialEq,Eq,Hash, knuffel::DecodeScalar )]
// pub struct Bind {
//     mode: Mode,
//     keys: Keys,
// }

#[derive(knus::Decode, Default, Debug, PartialEq, Serialize)]
pub struct Mode {
    press: Option<Vec<String>>,
    release: Option<Vec<String>>,
    repeat: Option<Vec<String>>,
}

fn parse_arg_node<S: knus::traits::ErrorSpan, T: knus::traits::DecodeScalar<S>>(
    name: &str,
    node: &knus::ast::SpannedNode<S>,
    ctx: &mut knus::decode::Context<S>,
) -> Result<T, DecodeError<S>> {
    let mut iter_args = node.arguments.iter();
    let val = iter_args.next().ok_or_else(|| {
        DecodeError::missing(node, format!("additional argument `{name}` is required"))
    })?;

    let value = knus::traits::DecodeScalar::decode(val, ctx)?;

    if let Some(val) = iter_args.next() {
        ctx.emit_error(DecodeError::unexpected(
            &val.literal,
            "argument",
            "unexpected argument",
        ));
    }
    for name in node.properties.keys() {
        ctx.emit_error(DecodeError::unexpected(
            name,
            "property",
            format!("unexpected property `{}`", name.escape_default()),
        ));
    }
    for child in node.children() {
        ctx.emit_error(DecodeError::unexpected(
            child,
            "node",
            format!("unexpected node `{}`", child.node_name.escape_default()),
        ));
    }

    Ok(value)
}

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
        Self::from_kdl(&path, &string)
    }
    pub fn from_kdl(path: &str, string: &str) -> Result<Self, MudrasError> {
        let res: Config = knus::parse(path, string).unwrap();
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    #[test]
    fn parse_config_file() -> Result<()> {
        Config::get()?;
        Ok(())
    }
}
