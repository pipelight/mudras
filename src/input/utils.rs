use evdev::{Device, KeyCode};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

pub fn check_device_is_keyboard(device: &Device) -> bool {
    if device
        .supported_keys()
        .is_some_and(|keys| keys.contains(KeyCode::KEY_ENTER))
    {
        if device.name() == Some("swhkd virtual output") {
            return false;
        }
        log::debug!("Keyboard: {}", device.name().unwrap(),);
        true
    } else {
        log::trace!("Other: {}", device.name().unwrap(),);
        false
    }
}

/// A struct that stores the keyboard current and n-1 state.
#[derive(Default, Debug, PartialEq, Serialize)]
pub struct KeyboardState {
    /// Current state
    current: State,
    /// Previous state (keystroke n-1)
    previous: State,
}

#[derive(Default, Debug, PartialEq, Serialize)]
pub struct State {
    keys: HashMap<KeyCode, State>,
}

#[derive(Default, Debug, PartialEq, Serialize)]
pub enum KeyState {
    Pressed,
    Released,
    #[default]
    Undefined,
}
