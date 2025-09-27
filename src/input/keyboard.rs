use super::utils::{self, KeyState, KeyboardState, SubmapState};
use super::virtuals;

use crate::config::{self, Bind, Config};
use crate::input::udev::handle_udev;
use crate::server::Server;

// Keyboard
use evdev::{AttributeSet, Device, EventStream, EventSummary, KeyCode};
use std::collections::HashMap;
use tokio::time::{sleep, Instant};
use tokio::{pin, select, sync::mpsc};
use tokio_stream::{StreamExt, StreamMap};
use tokio_udev::{AsyncMonitorSocket, EventType, MonitorBuilder};

// Error
use crate::error::{LibError, MudrasError, WrapError};
use miette::{Error, Result};
use tracing::{debug, error, info, trace, warn};

impl Server {
    pub async fn listen_keyboard(&self, config: &Config) -> Result<(), MudrasError> {
        // Collect keyboard devices
        let keyboard_devices: Vec<_> = evdev::enumerate()
            .filter(|(_path, device)| utils::check_device_is_keyboard(device))
            .collect();
        if keyboard_devices.is_empty() {
            let message = "No valid keyboard device was detected!";
            let help = "";
            let err = LibError::builder().msg(message).help(help).build();
            error!("{}", message);
        }

        debug!("{} keyboard device(s) detected.", keyboard_devices.len());

        // Collect pointer devices
        // (Not used!)
        let pointer_devices: Vec<_> = evdev::enumerate()
            .filter(|(_path, device)| utils::check_device_is_pointer(device))
            .collect();

        // Apparently, having a single uinput device with keys, relative axes and switches
        // prevents some libraries to listen to these events. The easy fix is to have separate
        // virtual devices, one for keys and relative axes (`uinput_device`) and another one
        // just for switches (`uinput_switches_device`).
        let mut virtual_keyboard = virtuals::create_keyboard()?;
        let mut virtual_pointer = virtuals::create_pointer()?;
        let mut virtual_switch = virtuals::create_switch()?;

        // Per keyboard State
        let mut keyboard_stream_map: StreamMap<String, EventStream> = StreamMap::new();
        let mut keyboard_states = HashMap::new();
        for (path, mut device) in keyboard_devices.into_iter() {
            let _ = device.grab();
            let path = path.to_str().unwrap();
            keyboard_stream_map.insert(path.to_string(), device.into_event_stream()?);
            keyboard_states.insert(path.to_string(), KeyboardState::default());
        }
        // Bindings state
        let submaps = config.get_submaps()?;
        let mut submaps_state = SubmapState {
            submaps,
            ..Default::default()
        };

        let mut udev =
            AsyncMonitorSocket::new(MonitorBuilder::new()?.match_subsystem("input")?.listen()?)?;

        loop {
            select! {
                // Device detection
                // Adds every keyboard to the global stream map.
                Some(Ok(event)) = udev.next() => {
                    handle_udev(event, &mut keyboard_stream_map)?;
                }
                Some((path, Ok(event))) = keyboard_stream_map.next() => {
                    match event.destructure() {
                        EventSummary::Key(_type, keycode, value) => {
                            let state = match value {
                                1 => KeyState::Pressed,
                                0 => KeyState::Released,
                                _ => KeyState::Undefined,
                            };
                            match state {
                                KeyState::Pressed | KeyState::Released => {
                                    // trace!("key={:#?},state={:#?}", keycode, state);

                                    if let Some(keyboard_state) = keyboard_states.get_mut(&path) {
                                        // Update keyboard representation state.
                                        keyboard_state.previous = keyboard_state.current.clone();
                                        for (key,value) in &keyboard_state.current.keys.clone() {
                                            if value == &KeyState::Released {
                                                keyboard_state.current.keys.remove(&key);
                                            }
                                        }
                                        keyboard_state.current.keys.insert(keycode,state);

                                        // Grab event if a modifier of the submap is pressed.
                                        let current_submap = submaps_state.current.clone();
                                        let binds = submaps_state.submaps.get(&current_submap).unwrap();
                                        let mods = config::utils::get_modifiers(binds).unwrap();
                                        // Test if any submap modifier is pressed/released.
                                        let has_any_mod = mods.iter().any(|modifier| keyboard_state.current.keys.contains_key(modifier));
                                        if !has_any_mod {
                                           virtual_keyboard.emit(&[event]).unwrap();
                                        }

                                        // Trigger action.
                                        utils::trigger_action(&mut submaps_state,keyboard_state).unwrap();

                                    }
                                }
                                _ => {}
                            }
                        }
                        EventSummary::Switch(_, _, _) => {
                            virtual_switch.emit(&[event]).unwrap();
                        }
                        _ => {
                            virtual_keyboard.emit(&[event]).unwrap();
                        }
                    };
                }
            }
        }
    }
}
