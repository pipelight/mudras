use crate::config::{Bind, Command, KeyAction, Keyword};
use pipelight_exec::Process;

use evdev::{AbsoluteAxisCode, Device, KeyCode};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

// Error
use crate::error::{LibError, MudrasError, WrapError};
use miette::{Error, Result};
use tracing::{debug, error, info, trace, warn};

pub fn check_device_is_keyboard(device: &Device) -> bool {
    if device
        .supported_keys()
        .is_some_and(|keys| keys.contains(KeyCode::KEY_ENTER))
    {
        if device.name() == Some("Mudras virtual keyboard") {
            return false;
        }
        true
    } else {
        false
    }
}
pub fn check_device_is_pointer(device: &Device) -> bool {
    if device
        .supported_absolute_axes()
        .is_some_and(|axes| axes.contains(AbsoluteAxisCode::ABS_X))
    {
        if device.name() == Some("Mudras virtual pointer") {
            return false;
        }
        true
    } else {
        false
    }
}

/// A struct that stores the keyboard current and n-1 state.
#[derive(Default, Debug, PartialEq, Serialize)]
pub struct KeyboardState {
    /// Current state
    pub current: State,
    /// Previous state (keystroke n-1)
    pub previous: State,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize)]
pub struct State {
    pub keys: HashMap<KeyCode, KeyState>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize)]
pub enum KeyState {
    Pressed,
    Released,
    #[default]
    Undefined,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SubmapState {
    pub submaps: HashMap<String, Vec<Bind>>,
    pub current: String,
}
impl Default for SubmapState {
    fn default() -> Self {
        Self {
            submaps: HashMap::new(),
            current: "main".to_owned(),
        }
    }
}

/// Compare keyboard state and binds.
pub fn trigger_action(
    submaps_state: &mut SubmapState,
    keyboard_state: &KeyboardState,
) -> Result<(), MudrasError> {
    // Sort current keyboard keys sequence
    let current_keys: Vec<KeyCode> = keyboard_state
        .current
        .keys
        .clone()
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    let mut sorted_current_keys = current_keys.clone();
    sorted_current_keys.sort();

    // info!("current = {:#?}", current_keys);
    let current_submap = submaps_state.current.clone();
    let binds = submaps_state.submaps.get(&current_submap).unwrap();
    let mut sorted_binds = binds.clone();
    sorted_binds.iter_mut().for_each(|e| e.sequence.sort());

    let current_keys_and_state: Vec<(KeyCode, KeyState)> =
        keyboard_state.current.keys.clone().into_iter().collect();

    for bind in binds {
        // Sort bind key sequence
        let sequence: Vec<KeyCode> = bind.sequence.clone();

        // info!("bind = {:#?}", sequence);
        let mut sorted_sequence = sequence.clone();
        sorted_sequence.sort();

        // A bind sequence is matched against the current keyboard keys
        let sorted_binds_keys: Vec<Vec<KeyCode>> =
            sorted_binds.iter().map(|e| e.sequence.clone()).collect();

        if sorted_binds_keys.contains(&sorted_current_keys) {
            // A bind seq is matched against current key states
            if let Some(action) = &bind.action.press {};
            if let Some(action) = &bind.action.release {};

            trace!("triggered bind = {:#?}", bind);
            if let Some(press) = &bind.action.press {
                if let Some(commands) = &press.commands {
                    for cmd in commands {
                        match cmd {
                            Command::Sh(stdin) => {
                                let mut p =
                                    Process::new().stdin(&stdin).term().background().to_owned();
                                p.run().unwrap();
                            }
                            Command::Internal(e) => match e {
                                Keyword::Enter(submap) => {}
                                Keyword::Exit => {}
                            },
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
