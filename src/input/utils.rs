use crate::config::{Bind, BindArgs, BindSequence, Binds, Command, Keyword, Submaps};
use pipelight_exec::Process;

use evdev::{uinput::VirtualDevice, AbsoluteAxisCode, Device, InputEvent, KeyCode};

use std::collections::HashMap;

// Error
use crate::error::{LibError, MudrasError, WrapError};
use miette::Result;
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
#[derive(Default, Debug, PartialEq)]
pub struct KeyboardState {
    /// Current state
    pub current: State,
    /// Previous state (keystroke n-1)
    pub previous: State,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct State {
    pub keys: HashMap<KeyCode, KeyState>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyState {
    Pressed,
    Released,
    #[default]
    Undefined,
}

#[derive(Clone, Debug)]
pub struct SubmapState {
    pub submaps: Submaps,
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
    key_state: &KeyState,

    virtual_keyboard: &mut VirtualDevice,
    event: InputEvent,
) -> Result<(), MudrasError> {
    // Get current submap
    let name = submaps_state.current.clone();
    if let Some(submap) = submaps_state.submaps.get(&name) {
        // Sort keyboard sequence for comparison
        let mut keyboard_sequence: Vec<(KeyCode, KeyState)> =
            keyboard_state.current.keys.clone().into_iter().collect();
        keyboard_sequence.sort_by(|a, b| a.0.cmp(&b.0));

        trace!("{:#?}", keyboard_sequence);
        // A bind sequence is matched against the current keyboard sequence
        if let Some(bind_args) = submap.binds.get(&keyboard_sequence) {
            // Extra step for release keys
            match key_state {
                KeyState::Released => {
                    if keyboard_state.previous.keys.len() > keyboard_state.current.keys.len() {
                        virtual_keyboard.emit(&[event]).unwrap();
                        return Ok(());
                    }
                }
                KeyState::Pressed => {}
                _ => {}
            }
            // Trigger action as soon as keys are detected.
            for cmd in &bind_args.commands {
                match cmd {
                    Command::Sh(stdin) => {
                        let mut p = Process::new().stdin(&stdin).term().background().to_owned();
                        p.run()?;
                    }
                    Command::Internal(e) => match e {
                        Keyword::Enter(submap_name) => {
                            submaps_state.current = submap_name.to_owned();
                        }
                        Keyword::Exit => {
                            submaps_state.current = "main".to_owned();
                        }
                    },
                }
            }
        } else {
            virtual_keyboard.emit(&[event]).unwrap();
        }
    } else {
        virtual_keyboard.emit(&[event]).unwrap();
    }
    Ok(())
}
