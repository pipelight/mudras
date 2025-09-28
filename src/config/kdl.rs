/*
* KDL parsing utilities.
*
* Please keep in mind that structs here have nothing in common
* with the configuration file structure (node, children...)
* The file has a "free" form that is processed from inside the decode_children functions,
* then content is parsed into final easily usable structs.
*
*/

use super::utils::{self, get_modifiers};
use crate::input::utils::KeyState;
use evdev::KeyCode;
// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Parse
use knus::errors::DecodeError;
use knus::{Decode, DecodeChildren};

// Error
use crate::error::{MudrasError, WrapError};
use miette::Result;
use tracing::{error, trace};

#[derive(Debug, Clone)]
pub struct Config {
    pub submaps: Submaps,
}

impl<S> knus::DecodeChildren<S> for Config
where
    S: knus::traits::ErrorSpan,
{
    fn decode_children(
        nodes: &[knus::ast::SpannedNode<S>],
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        // Create a main submap
        let mut main: Submap = Submap {
            name: "main".to_owned(),
            mods: vec![],
            binds: Binds::new(),
        };
        let mut raw_binds: Binds = HashMap::new();
        let mut binds: Binds = HashMap::new();

        let mut submaps = Submaps::new();

        for node in nodes {
            match &*node.node_name.to_string() {
                "@submap" => {
                    let submap = Submap::decode_node(node, ctx)?;
                    submaps.insert(submap.name.clone(), submap);
                }
                _ => {
                    // Modifiers
                    let bind = Bind::decode_node(node, ctx)?;
                    raw_binds.insert(bind.sequence, bind.args);

                    // Binds
                    let bind = Bind::decode_node(node, ctx)?;
                    // Sort sequence
                    let mut sequence = bind.sequence.clone();
                    sequence.sort_by(|a, b| a.0.cmp(&b.0));
                    binds.insert(sequence, bind.args);
                }
            };
        }
        // Main submap
        let mods = get_modifiers(&raw_binds).unwrap();
        main.mods = mods;
        main.binds = binds;

        submaps.insert(main.name.clone(), main);
        Ok(Self { submaps })
    }
}

pub type Submaps = HashMap<String, Submap>;

#[derive(Default, Clone, Debug)]
pub struct Submap {
    pub name: String,
    pub mods: Vec<KeyCode>,
    pub binds: Binds,
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
        // Modifiers
        let mut raw_binds: Binds = HashMap::new();
        for node in node.children() {
            let bind = Bind::decode_node(node, ctx)?;
            raw_binds.insert(bind.sequence, bind.args);
        }
        let mods = get_modifiers(&raw_binds).unwrap();

        // Binds
        let mut binds: Binds = HashMap::new();
        for node in node.children() {
            let bind = Bind::decode_node(node, ctx)?;
            // Sort sequence
            let mut sequence = bind.sequence.clone();
            sequence.sort_by(|a, b| a.0.cmp(&b.0));

            binds.insert(sequence, bind.args);
        }

        Ok(Submap { name, mods, binds })
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum SequenceType {
    Press,
    Release,
}

pub type Binds = HashMap<BindSequence, BindArgs>;

#[derive(Debug, Clone, PartialEq)]
pub struct Bind {
    pub sequence: BindSequence,
    pub args: BindArgs,
}

pub type BindSequence = Vec<(KeyCode, KeyState)>;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct BindArgs {
    pub commands: Vec<Command>,
    // Extra properties
    /// Whether the keys must be passed to underlying applications.
    /// Default to true (keys are not passed).
    pub swallow: bool,
    /// Repeat the action when key maintained.
    /// Default to true.
    pub repeat: bool,
    /// Do not trigger release action when too much key pressed.
    /// Default to true.
    pub exact: bool,
}

impl<S> knus::Decode<S> for Bind
where
    S: knus::traits::ErrorSpan,
{
    fn decode_node(
        node: &knus::ast::SpannedNode<S>,
        ctx: &mut knus::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        // Binds
        // Bind args
        let mut args = BindArgs {
            swallow: true,
            repeat: false,
            exact: true,
            commands: vec![],
        };

        // Sequence Global args
        for (name, val) in &node.properties {
            match &***name {
                "swallow" => {
                    args.swallow = knus::traits::DecodeScalar::decode(val, ctx)?;
                }
                _ => {}
            }
        }

        // Bind sequence
        let mut sequence = vec![];
        for child in node.children() {
            let name: String = child.node_name.to_string();
            match &*name {
                "@press" => {
                    sequence =
                        utils::bind_to_keys(&node.node_name.to_string(), &SequenceType::Press)
                            .unwrap();
                    let mut iter_args = node.arguments.iter();
                    if let Some(val) = iter_args.next() {
                        args.repeat = knus::traits::DecodeScalar::decode(val, ctx)?
                    };
                    args.commands = children_to_commands(child, ctx)?;
                }
                "@release" => {
                    sequence =
                        utils::bind_to_keys(&node.node_name.to_string(), &SequenceType::Release)
                            .unwrap();
                    args.commands = children_to_commands(child, ctx)?;
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

        Ok(Self { sequence, args })
    }
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
