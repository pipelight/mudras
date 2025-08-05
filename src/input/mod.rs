mod utils;

// Keyboard
use evdev::{AttributeSet, Device, EventType, KeyCode};

use tokio::time::{sleep, Instant};
use tokio::{select, sync::mpsc};
use tokio_stream::{StreamExt, StreamMap};
use tokio_udev::{AsyncMonitorSocket, EventType, MonitorBuilder};

use std::path::PathBuf;

// Error
use crate::error::{LibError, MudrasError, WrapError};
use log::{error, trace};
use miette::{Error, IntoDiagnostic, Result};

pub async fn listen_keyboard() -> Result<(), MudrasError> {
    let keyboard_devices: Vec<_> = evdev::enumerate().collect();
    if keyboard_devices.is_empty() {
        let message = "No valid keyboard device was detected!";
        let help = "";
        error!("{}", message);
        let err = LibError::builder().msg(message).help(help).build();
    }
    log::debug!("{} Keyboard device(s) detected.", keyboard_devices.len());

    let mut keyboard_stream_map = StreamMap::new();
    for (path, mut device) in keyboard_devices.into_iter() {
        let _ = device.grab();
        let path = match path.to_str() {
            Some(p) => p,
            None => {
                continue;
            }
        };
        keyboard_stream_map.insert(path.to_string(), device.into_event_stream()?);
    }

    loop {
        keyboard_stream_map.next().await;

        // select! {
        // Some((node, Ok(event))) = keyboard_stream_map.next().await => {
        // let key = match event.kind() {
        //     InputEventKind::Key(keycode) => keycode,
        //     _=>{}
        // };
        // }
        // }
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
