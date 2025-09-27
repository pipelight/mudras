use super::utils;

// Keyboard
use evdev::{Device, EventStream, EventSummary, KeyCode};
use std::collections::HashMap;

use tokio_stream::{StreamExt, StreamMap};
use tokio_udev::{AsyncMonitorSocket, Event as UdevEvent, EventType, MonitorBuilder};

// Error
use crate::error::{LibError, MudrasError, WrapError};
use miette::{Error, Result};
use tracing::{debug, error, info, trace, warn};

pub fn handle_udev(
    event: UdevEvent,
    keyboard_stream_map: &mut StreamMap<String, EventStream>,
) -> Result<(), MudrasError> {
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
                        let _ = device.grab();
                        keyboard_stream_map.insert(node.to_string(), device.into_event_stream()?);
                        info!("added keyboard device '{}' at '{}'.", name, node);
                    } else if utils::check_device_is_pointer(&device) {
                        info!("added pointer device '{}' at '{}'.", name, node);
                    }
                }
            }
        }
        EventType::Remove => {
            if let Some(path) = event.devnode() {
                let node = path.to_str().unwrap();
                if keyboard_stream_map.contains_key(node) {
                    let stream = keyboard_stream_map
                        .remove(node)
                        .expect("device not in stream_map");
                    let name = stream.device().name().unwrap_or("[unknown]");
                    info!("removed device '{}' at '{}'.", name, node);
                }
            }
        }
        _ => {
            trace!("ignored udev event of type: {:?}", event.event_type());
        }
    }
    Ok(())
}
