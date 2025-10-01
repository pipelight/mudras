use super::Server;
use crate::input::utils;

// Signals
use signal_hook::consts::signal::*;
// use signal_hook::iterator::SignalsInfo;
// use signal_hook_tokio::SignalsInfo;
use signal_hook_async_std::SignalsInfo;
use std::process::exit;

use futures::StreamExt;
use tokio::select;

// Error
use crate::error::{LibError, MudrasError, WrapError};
use miette::Result;
use tracing::{debug, error, info, trace, warn};

impl Server {
    pub async fn handle_signals(&self) -> Result<(), MudrasError> {
        let mut signals: SignalsInfo = SignalsInfo::new([
            SIGUSR1, SIGUSR2, SIGHUP, SIGABRT, SIGBUS, SIGCONT, SIGINT, SIGPIPE, SIGQUIT, SIGSYS,
            SIGTERM, SIGTRAP, SIGTSTP, SIGVTALRM, SIGXCPU, SIGXFSZ,
        ])?;

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
                        SIGHUP => {
                            // Update configuration

                        }
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
            }
        }
    }
}
