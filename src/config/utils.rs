use super::{Bind, Binds, SequenceType};

use evdev::KeyCode;
use std::path::Path;

// Error
use crate::{
    error::{LibError, MudrasError},
    input::utils::KeyState,
};
use miette::Result;
use tracing::{error, trace};

pub fn get_modifiers(binds: &Binds) -> Result<Vec<KeyCode>, MudrasError> {
    let mut mods: Vec<KeyCode> = vec![];
    for (sequence, args) in binds.clone() {
        if let Some(key) = sequence.first() {
            mods.push(key.0);
        }
    }
    mods.sort();
    mods.dedup();
    Ok(mods)
}

/// Expand tild "~" in file path.
pub fn shellexpand(relpath: &str) -> Result<String, MudrasError> {
    let source: String = match relpath.starts_with("~") {
        false => relpath.to_owned(),
        true => relpath.replace("~", dirs::home_dir().unwrap().to_str().unwrap()),
    };

    let path = Path::new(&source);
    if path.exists() {
        Ok(source)
    } else {
        let message = format!("Couldn't find file {:#?} expended to {:#?}.", relpath, path);
        error!("{:#?}", message);
        let err = LibError::builder()
            .msg(&message)
            .help("Are you sure the file exist?")
            .build();
        return Err(err.into());
    }
}

/// Transform a bind definition into its keycode.
pub fn bind_to_keys(
    bind: &str,
    sequence_type: &SequenceType,
) -> Result<Vec<(KeyCode, KeyState)>, MudrasError> {
    let keys: Vec<&str> = bind.split("+").collect();
    let mut sequence = vec![];

    for key in keys {
        let keycode = match &*key.to_lowercase() {
            "escape" => evdev::KeyCode::KEY_ESC,

            "super" => KeyCode::KEY_LEFTMETA,
            "super_l" => KeyCode::KEY_LEFTMETA,
            "super_r" => KeyCode::KEY_RIGHTMETA,

            "ctrl" => evdev::KeyCode::KEY_LEFTCTRL,
            "ctrl_l" => evdev::KeyCode::KEY_LEFTCTRL,
            "ctrl_r" => evdev::KeyCode::KEY_RIGHTCTRL,

            "shift" => evdev::KeyCode::KEY_LEFTSHIFT,
            "shift_l" => evdev::KeyCode::KEY_LEFTSHIFT,
            "shift_r" => evdev::KeyCode::KEY_RIGHTSHIFT,

            "alt" => evdev::KeyCode::KEY_LEFTALT,
            "tab" => evdev::KeyCode::KEY_TAB,
            "backspace" => evdev::KeyCode::KEY_BACKSPACE,

            "enter" => evdev::KeyCode::KEY_ENTER,
            "space" => evdev::KeyCode::KEY_SPACE,

            "left" => evdev::KeyCode::KEY_LEFT,
            "down" => evdev::KeyCode::KEY_DOWN,
            "up" => evdev::KeyCode::KEY_UP,
            "right" => evdev::KeyCode::KEY_RIGHT,

            "1" => evdev::KeyCode::KEY_1,
            "2" => evdev::KeyCode::KEY_2,
            "3" => evdev::KeyCode::KEY_3,
            "4" => evdev::KeyCode::KEY_4,
            "5" => evdev::KeyCode::KEY_5,
            "6" => evdev::KeyCode::KEY_6,
            "7" => evdev::KeyCode::KEY_7,
            "8" => evdev::KeyCode::KEY_8,
            "9" => evdev::KeyCode::KEY_9,
            "0" => evdev::KeyCode::KEY_0,

            "q" => evdev::KeyCode::KEY_Q,
            "w" => evdev::KeyCode::KEY_W,
            "e" => evdev::KeyCode::KEY_E,
            "r" => evdev::KeyCode::KEY_R,
            "t" => evdev::KeyCode::KEY_T,
            "y" => evdev::KeyCode::KEY_Y,
            "u" => evdev::KeyCode::KEY_U,
            "i" => evdev::KeyCode::KEY_I,
            "o" => evdev::KeyCode::KEY_O,
            "p" => evdev::KeyCode::KEY_P,

            "a" => evdev::KeyCode::KEY_A,
            "s" => evdev::KeyCode::KEY_S,
            "d" => evdev::KeyCode::KEY_D,
            "f" => evdev::KeyCode::KEY_F,
            "g" => evdev::KeyCode::KEY_G,
            "h" => evdev::KeyCode::KEY_H,
            "j" => evdev::KeyCode::KEY_J,
            "k" => evdev::KeyCode::KEY_K,
            "l" => evdev::KeyCode::KEY_L,

            "z" => evdev::KeyCode::KEY_Z,
            "x" => evdev::KeyCode::KEY_X,
            "c" => evdev::KeyCode::KEY_C,
            "v" => evdev::KeyCode::KEY_V,
            "b" => evdev::KeyCode::KEY_B,
            "n" => evdev::KeyCode::KEY_N,
            "m" => evdev::KeyCode::KEY_M,

            _ => evdev::KeyCode::KEY_RESERVED,
        };
        sequence.push((keycode, KeyState::Pressed));
    }

    match &sequence_type {
        SequenceType::Release => {
            if let Some((_key, ref mut state)) = sequence.iter_mut().last() {
                *state = KeyState::Released;
            }
        }
        _ => {}
    };
    Ok(sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_sequence_to_keys() -> Result<()> {
        let res = bind_to_keys("", &SequenceType::Press)?;
        let empty: Vec<(KeyCode, KeyState)> = vec![(KeyCode::KEY_RESERVED, KeyState::Pressed)];
        assert_eq!(empty, res);

        let res = bind_to_keys("Super", &SequenceType::Release)?;
        assert_eq!(vec![(KeyCode::KEY_LEFTMETA, KeyState::Released)], res);

        let res = bind_to_keys("Super+T", &SequenceType::Press)?;
        assert_eq!(
            vec![
                (KeyCode::KEY_LEFTMETA, KeyState::Pressed),
                (KeyCode::KEY_T, KeyState::Pressed)
            ],
            res
        );
        Ok(())
    }
}
