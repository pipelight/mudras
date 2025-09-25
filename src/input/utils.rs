use crate::config::{Bind, Command, Keyword};
use pipelight_exec::Process;

use evdev::{Device, KeyCode};
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
        if device.name() == Some("mudras virtual output") {
            return false;
        }
        debug!("Keyboard: {}", device.name().unwrap());
        true
    } else {
        trace!("Other: {}", device.name().unwrap());
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

/// Compare keybard state and binds
pub fn do_trigger_press(
    binds: Vec<&Bind>,
    keyboard_state: &KeyboardState,
) -> Result<(), MudrasError> {
    // Sort current keyboard keys sequence
    let mut current_keys: Vec<KeyCode> = keyboard_state
        .current
        .keys
        .iter()
        .map(|(k, _)| k.to_owned())
        .collect();
    current_keys.sort();
    // info!("current = {:#?}", current_keys);

    for bind in binds {
        // Sort bind key sequence
        let mut sequence: Vec<KeyCode> = bind.sequence.clone();
        sequence.sort();
        // info!("bind = {:#?}", sequence);

        if sequence == current_keys {
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
