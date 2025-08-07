mod uinput;
mod utils;

use std::path::PathBuf;

// Signals
use signal_hook::consts::signal::*;
// use signal_hook::iterator::SignalsInfo;
// use signal_hook_tokio::SignalsInfo;
use signal_hook_async_std::SignalsInfo;
use std::process::exit;

// Keyboard
use evdev::{AttributeSet, Device, EventStream, EventSummary, KeyCode};
use std::collections::HashMap;
use tokio::time::{sleep, Instant};
use tokio::{pin, select, sync::mpsc};
use tokio_stream::{StreamExt, StreamMap};
use tokio_udev::{AsyncMonitorSocket, EventType, MonitorBuilder};

use utils::{KeyState, KeyboardState};

// Error
use crate::error::{LibError, MudrasError, WrapError};
use log::{debug, error, info, trace, warn};
use miette::{Error, IntoDiagnostic, Result};

pub async fn listen_keyboard() -> Result<(), MudrasError> {
    let mut signals: SignalsInfo = SignalsInfo::new([
        SIGUSR1, SIGUSR2, SIGHUP, SIGABRT, SIGBUS, SIGCONT, SIGINT, SIGPIPE, SIGQUIT, SIGSYS,
        SIGTERM, SIGTRAP, SIGTSTP, SIGVTALRM, SIGXCPU, SIGXFSZ,
    ])?;

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
        let path = match path.to_str() {
            Some(p) => p,
            None => {
                continue;
            }
        };
        keyboard_stream_map.insert(path.to_string(), device.into_event_stream()?);
        keyboard_states.insert(path.to_string(), KeyboardState::default());
    }

    loop {
        select! {
            Some(signal) = signals.next() => {
                match signal {
                    SIGUSR1 => {
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(utils::check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }
                    }
                    SIGUSR2 => {
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(utils::check_device_is_keyboard) {
                            let _ = device.grab();
                        }
                    }
                    SIGHUP => {}
                    SIGINT => {
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(utils::check_device_is_keyboard) {
                            let _ = device.ungrab();
                        }
                        warn!("Received SIGINT signal, exiting...");
                        exit(1);
                    }
                    _ => {
                        for mut device in evdev::enumerate().map(|(_, device)| device).filter(utils::check_device_is_keyboard) {
                           let _ = device.ungrab();
                        }
                        warn!("Received signal: {:#?}", signal);
                        warn!("Exiting...");
                        exit(1);
                    }
                }
            }

            // Device detection
            Some(Ok(event)) = udev.next() => {
                if !event.is_initialized() {
                    warn!("Received udev event with uninitialized device.");
                }

                let node = match event.devnode() {
                    None => { continue; },
                    Some(node) => {
                        match node.to_str() {
                            None => { continue; },
                            Some(node) => node,
                        }
                    },
                };

                match event.event_type() {
                    EventType::Add => {
                        let mut device = match Device::open(node) {
                            Err(e) => {
                                error!("Could not open evdev device at {}: {}", node, e);
                                continue;
                            },
                            Ok(device) => device
                        };
                        let name = device.name().unwrap_or("[unknown]").to_string();
                        if utils::check_device_is_keyboard(&device) {
                            info!("Device '{}' at '{}' added.", name, node);
                            let _ = device.grab();
                            keyboard_stream_map.insert(node.to_string(), device.into_event_stream()?);
                        }
                    }
                    EventType::Remove => {
                        if keyboard_stream_map.contains_key(node) {
                            let stream = keyboard_stream_map.remove(node).expect("device not in stream_map");
                            let name = stream.device().name().unwrap_or("[unknown]");
                            info!("Device '{}' at '{}' removed", name, node);
                        }
                    }
                    _ => {
                        trace!("Ignored udev event of type: {:?}", event.event_type());
                    }
                }
            }
            Some((node, Ok(event))) = keyboard_stream_map.next() => {
                match event.destructure() {
                    EventSummary::Switch(_, _, _) => {
                        uinput_switches_device.emit(&[event]).unwrap();
                        continue
                    }
                    EventSummary::Key(_type, keycode, value) => {
                        match value {
                            // Key press
                            1 => {
                                let state = KeyState::Pressed;
                                trace!("key={:#?},state={:#?}", keycode, state);
                            }
                            // Key release
                            0 => {
                                let state = KeyState::Released;
                                trace!("key={:#?},state={:#?}", keycode, state);
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        uinput_device.emit(&[event]).unwrap();
                        continue
                    }
                };
                uinput_device.emit(&[event]).unwrap();
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    #[tokio::test]
    async fn test_listen_keyboard_events() -> Result<()> {
        listen_keyboard().await?;
        Ok(())
    }
}
