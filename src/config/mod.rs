mod utils;

// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Parse
use knus::errors::DecodeError;
use knus::{Decode, DecodeChildren};

// Error
use crate::error::{MudrasError, WrapError};
use log::{error, trace};
use miette::{Error, IntoDiagnostic, Result};

#[derive(Debug, PartialEq, Serialize)]
pub struct Config {
    binds: Vec<Items>,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum Items {
    Bind(Bind),
    Submap(Submap),
}

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
            match &*node.node_name.to_string() {
                "@submap" => {
                    let submap = Submap::decode_node(node, ctx)?;
                    // println!("{:#?}", submap);
                    binds.push(Items::Submap(submap));
                }
                _ => {
                    let bind = Bind::decode_node(node, ctx)?;
                    // println!("{:#?}", bind);
                    binds.push(Items::Bind(bind));
                }
            };
        }
        Ok(Self { binds })
    }
}

#[derive(Default, Debug, PartialEq, Serialize)]
pub struct Submap {
    pub name: String,
    pub binds: Vec<Bind>,
}
impl<S> knus::Decode<S> for Submap
where
    S: knus::traits::ErrorSpan,
{
    fn decode_node(
        node: &knus::ast::SpannedNode<S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        // Global props
        let mut name: String = "".to_owned();
        for (key, val) in &node.properties {
            match &***key {
                "name" => {
                    name = knus::traits::DecodeScalar::decode(val, ctx)?;
                }
                _ => {}
            };
        }
        // Binds
        let mut binds = Vec::new();
        for node in node.children() {
            let bind = Bind::decode_node(node, ctx)?;
            binds.push(bind);
        }
        Ok(Submap { name, binds })
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Bind {
    pub key: String,
    // #[serde(flatten)]
    pub action: Action,
    // Exta properties
    /// Whether the keys must be passed to underlying applications
    /// Default to true (keys are not passed)
    pub swallow: Option<bool>,
}
impl Default for Bind {
    fn default() -> Self {
        Self {
            key: String::default(),
            action: Action::default(),
            swallow: Some(true),
        }
    }
}
impl<S> knus::Decode<S> for Bind
where
    S: knus::traits::ErrorSpan,
{
    fn decode_node(
        node: &knus::ast::SpannedNode<S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let key = node.node_name.to_string();

        // Global args
        let mut swallow = Some(true); // default
        for (name, val) in &node.properties {
            match &***name {
                "swallow" => {
                    swallow = Some(knus::traits::DecodeScalar::decode(val, ctx)?);
                }
                _ => {}
            }
        }

        let mut action = Action::default();
        for child in node.children() {
            let name: String = child.node_name.to_string();
            match &*name {
                "@press" => {
                    let mut iter_args = node.arguments.iter();
                    let repeat = if let Some(val) = iter_args.next() {
                        // println!("{:#?}", val);
                        knus::traits::DecodeScalar::decode(val, ctx)?
                    } else {
                        Some(true)
                    };

                    let commands = Some(children_to_commands(child, ctx)?);
                    action.press = Some(Press { repeat, commands });
                }
                "@release" => {
                    let commands = Some(children_to_commands(child, ctx)?);
                    action.release = Some(Release { commands });
                }
                _ => {
                    ctx.emit_error(DecodeError::unexpected(
                        child,
                        "node",
                        "only @press and @release or accepted",
                    ));
                }
            };
        }

        Ok(Self {
            key,
            action,
            swallow,
        })
    }
}

#[derive(knus::Decode, Debug, Default, PartialEq, Hash, Serialize)]
pub struct Action {
    #[knus(child)]
    pub press: Option<Press>,
    #[knus(child)]
    pub release: Option<Release>,
}

#[derive(knus::Decode, Debug, PartialEq, Hash, Serialize)]
pub struct Press {
    #[knus(property)]
    pub repeat: Option<bool>,
    // #[knus(children)]
    pub commands: Option<Vec<String>>,
}
impl Default for Press {
    fn default() -> Self {
        Self {
            repeat: Some(false),
            commands: None,
        }
    }
}

#[derive(knus::Decode, Debug, Default, PartialEq, Hash, Serialize)]
pub struct Release {
    pub commands: Option<Vec<String>>,
}

pub fn children_to_commands<S: knus::traits::ErrorSpan>(
    node: &knus::ast::SpannedNode<S>,
    ctx: &mut knus::decode::Context<S>,
) -> Result<Vec<String>, DecodeError<S>> {
    let mut commands: Vec<String> = vec![];
    for child in node.children() {
        if child.node_name.to_string() == "-" {
            let cmd: String = parse_arg_node("-", child, ctx)?;
            commands.push(cmd);
        }
    }
    Ok(commands)
}

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
    fn release_path() -> Result<PathBuf, MudrasError> {
        let mut path = PathBuf::new();
        path.push(utils::shellexpand("/home/anon/.config/mudras/config.kdl")?);
        Ok(path)
    }
    /// Return configuration from default file path.
    pub fn get() -> Result<Self, MudrasError> {
        // let config = CONFIG.lock().unwrap().clone();
        // Ok(config)

        #[cfg(debug_assertions)]
        let path = Self::debug_path();

        #[cfg(not(debug_assertions))]
        let path = Self::release_path()?;

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
        let res: Config = knus::parse(path, string)
            .map_err(miette::Report::new)
            .unwrap();
        Ok(res)
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
