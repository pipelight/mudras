use super::uinput;
use super::utils::{self, KeyState, KeyboardState};
use crate::config::{Bind, Config, Items};
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
        let keyboard_devices: Vec<_> = evdev::enumerate().collect();
        if keyboard_devices.is_empty() {
            let message = "No valid keyboard device was detected!";
            let help = "";
            error!("{}", message);
            let err = LibError::builder().msg(message).help(help).build();
        }
        debug!("{} Keyboard device(s) detected.", keyboard_devices.len());

        // Apparently, having a single uinput device with keys, relative axes and switches
        // prevents some libraries to listen to these events. The easy fix is to have separate
        // virtual devices, one for keys and relative axes (`uinput_device`) and another one
        // just for switches (`uinput_switches_device`).
        let mut uinput_device = match uinput::create_uinput_device() {
            Ok(dev) => dev,
            Err(e) => {
                let message = format!("Failed to create uinput device: \nErr: {:#?}", e);
                let help = "";
                error!("{}", message);
                let err = LibError::builder().msg(&message).help(help).build();
                return Err(err.into());
            }
        };
        let mut uinput_switches_device = match uinput::create_uinput_switches_device() {
            Ok(dev) => dev,
            Err(e) => {
                let message = format!("Failed to create uinput switches device: \nErr: {:#?}", e);
                let help = "";
                error!("{}", message);
                let err = LibError::builder().msg(&message).help(help).build();
                return Err(err.into());
            }
        };

        let mut udev =
            AsyncMonitorSocket::new(MonitorBuilder::new()?.match_subsystem("input")?.listen()?)?;

        let mut keyboard_stream_map: StreamMap<String, EventStream> = StreamMap::new();
        let mut keyboard_states = HashMap::new();

        for (path, mut device) in keyboard_devices.into_iter() {
            let _ = device.grab();
            let path = path.to_str().unwrap();
            keyboard_stream_map.insert(path.to_string(), device.into_event_stream()?);
            keyboard_states.insert(path.to_string(), KeyboardState::default());
        }

        // Match against bind
        let mut binds: Vec<&Bind> = vec![];
        for bind in &config.binds {
            match bind {
                Items::Bind(v) => {
                    binds.push(v);
                }
                Items::Submap(v) => {}
            };
        }

        loop {
            select! {
                // Device detection
                // Adds every keyboard to the global stream map.
                Some(Ok(event)) = udev.next() => {

                    if !event.is_initialized() {
                        warn!("Received udev event with uninitialized device.");
                    }

                    match event.event_type() {
                        EventType::Add => {
                            if let Some(path) = event.devnode() {
                                let node = path.to_str().unwrap();
                                if let Some(mut device) = Device::open(node).ok() {
                                    let name = device.name().unwrap_or("[unknown]").to_string();
                                    if utils::check_device_is_keyboard(&device) {
                                        info!("Device '{}' at '{}' added.", name, node);
                                        let _ = device.grab();
                                        keyboard_stream_map.insert(node.to_string(), device.into_event_stream()?);
                                    }
                                }
                            }
                        }
                        EventType::Remove => {
                            if let Some(path) = event.devnode() {
                                let node = path.to_str().unwrap();
                                if keyboard_stream_map.contains_key(node) {
                                    let stream = keyboard_stream_map.remove(node).expect("device not in stream_map");
                                    let name = stream.device().name().unwrap_or("[unknown]");
                                    info!("Device '{}' at '{}' removed", name, node);
                                }
                            }
                        }
                        _ => {
                            trace!("Ignored udev event of type: {:?}", event.event_type());
                        }
                    }
                }
                Some((path, Ok(event))) = keyboard_stream_map.next() => {
                    match event.destructure() {
                        EventSummary::Switch(_, _, _) => {
                            uinput_switches_device.emit(&[event]).unwrap();
                        }
                        EventSummary::Key(_type, keycode, value) => {
                            match value {
                                // Key press
                                1 => {
                                    let state = KeyState::Pressed;
                                    trace!("key={:#?},state={:#?}", keycode, state);

                                    if let Some(keyboard_state) = keyboard_states.get_mut(&path) {
                                        // Update keyboard representation state.
                                        keyboard_state.previous = keyboard_state.current.clone();
                                        for (key,value) in &keyboard_state.current.keys.clone() {
                                            if value == &KeyState::Released {
                                                keyboard_state.current.keys.remove(&key);
                                            }
                                        }
                                        keyboard_state.current.keys.insert(keycode,state);
                                        // Trigger binding
                                        utils::do_trigger_press(binds.clone(),keyboard_state).unwrap();
                                    }
                                }
                                // Key release
                                0 => {
                                    let state = KeyState::Released;
                                    trace!("key={:#?},state={:#?}", keycode, state);

                                    if let Some(keyboard_state) = keyboard_states.get_mut(&path) {
                                        // Update keyboard representation state.
                                        keyboard_state.previous = keyboard_state.current.clone();
                                        for (key,value) in &keyboard_state.current.keys.clone() {
                                            if value == &KeyState::Released {
                                                keyboard_state.current.keys.remove(&key);
                                            }
                                        }
                                        keyboard_state.current.keys.insert(keycode,state);
                                        // Trigger binding
                                        // utils::do_trigger_release(binds.clone(),keyboard_state).unwrap();
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    };

                    uinput_device.emit(&[event]).unwrap();
                }
            }
        }
    }
}
