use super::utils;

use evdev::KeyCode;
// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Parse
use knus::errors::DecodeError;
use knus::{Decode, DecodeChildren};

// Error
use crate::error::{MudrasError, WrapError};
use miette::{Error, IntoDiagnostic, Result};
use tracing::{error, trace};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Config {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Item {
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
        let mut items = Vec::new();
        for node in nodes {
            match &*node.node_name.to_string() {
                "@submap" => {
                    let submap = Submap::decode_node(node, ctx)?;
                    // println!("{:#?}", submap);
                    items.push(Item::Submap(submap));
                }
                _ => {
                    let bind = Bind::decode_node(node, ctx)?;
                    // println!("{:#?}", bind);
                    items.push(Item::Bind(bind));
                }
            };
        }
        Ok(Self { items })
    }
}

#[derive(Default, Clone, Debug, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bind {
    pub sequence: Vec<KeyCode>,
    // #[serde(flatten)]
    pub action: KeyAction,
    // Extra properties
    /// Whether the keys must be passed to underlying applications
    /// Default to true (keys are not passed)
    pub swallow: Option<bool>,
}
impl Default for Bind {
    fn default() -> Self {
        Self {
            sequence: utils::bind_to_keys(&String::default()).unwrap(),
            action: KeyAction::default(),
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
        let sequence = utils::bind_to_keys(&node.node_name.to_string()).unwrap();

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

        let mut action = KeyAction::default();
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
            sequence,
            action,
            swallow,
        })
    }
}

#[derive(knus::Decode, Debug, Clone, Default, PartialEq, Hash, Serialize)]
pub struct KeyAction {
    #[knus(child)]
    pub press: Option<Press>,
    #[knus(child)]
    pub release: Option<Release>,
}

#[derive(knus::Decode, Debug, Clone, PartialEq, Hash, Serialize)]
pub struct Press {
    #[knus(property)]
    pub repeat: Option<bool>,
    // #[knus(children)]
    pub commands: Option<Vec<Command>>,
}
impl Default for Press {
    fn default() -> Self {
        Self {
            repeat: Some(false),
            commands: None,
        }
    }
}

#[derive(knus::Decode, Debug, Clone, Default, PartialEq, Hash, Serialize)]
pub struct Release {
    pub commands: Option<Vec<Command>>,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Command {
    /// To be executed from a terminal
    Sh(String),
    /// Mudras internal special command
    Internal(Keyword),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    Enter(String),
    Exit,
}

pub fn children_to_commands<S: knus::traits::ErrorSpan>(
    node: &knus::ast::SpannedNode<S>,
    ctx: &mut knus::decode::Context<S>,
) -> Result<Vec<Command>, DecodeError<S>> {
    let mut commands: Vec<Command> = vec![];
    for child in node.children() {
        if child.node_name.to_string() == "-" {
            // Common bash instruction.
            let cmd: String = parse_arg_node("-", child, ctx)?;
            commands.push(Command::Sh(cmd));
        } else if child.node_name.to_string() == "@enter" {
            // Special keyword for entering submap.
            let value = child.arguments.first().unwrap();
            let submap_name = knus::traits::DecodeScalar::decode(value, ctx)?;
            commands.push(Command::Internal(Keyword::Enter(submap_name)));
        } else if child.node_name.to_string() == "@exit" {
            // Special keyword for exiting submap.
            commands.push(Command::Internal(Keyword::Exit));
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
