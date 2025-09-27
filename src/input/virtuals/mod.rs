mod constants;
use constants::*;

use evdev::{
    uinput::VirtualDevice, AbsInfo, AbsoluteAxisCode, AttributeSet, KeyCode, RelativeAxisCode,
    SwitchCode, UinputAbsSetup,
};

// Error
use crate::error::{LibError, MudrasError};
use miette::Result;
use tracing::{debug, error, info, trace, warn};

pub fn create_keyboard() -> Result<VirtualDevice, MudrasError> {
    let keys: AttributeSet<KeyCode> = get_all_keys().iter().copied().collect();
    let device = VirtualDevice::builder()?
        .name("Mudras virtual keyboard")
        .with_keys(&keys)?
        .build();
    match device {
        Ok(device) => Ok(device),
        Err(e) => {
            let message = format!("Failed to create uinput device: \nErr: {:#?}", e);
            let help = "";
            error!("{}", message);
            let err = LibError::builder().msg(&message).help(help).build();
            return Err(err.into());
        }
    }
}

pub fn create_switch() -> Result<VirtualDevice, MudrasError> {
    let switches: AttributeSet<SwitchCode> = get_all_switches().iter().copied().collect();

    let device = VirtualDevice::builder()?
        .name("Mudras virtual switch")
        .with_switches(&switches)?
        .build();
    match device {
        Ok(device) => Ok(device),
        Err(e) => {
            let message = format!("Failed to create uinput device: \nErr: {:#?}", e);
            let help = "";
            error!("{}", message);
            let err = LibError::builder().msg(&message).help(help).build();
            return Err(err.into());
        }
    }
}

/// Deprecated!
pub fn create_pointer() -> Result<VirtualDevice, MudrasError> {
    let relative_axes: AttributeSet<RelativeAxisCode> =
        get_all_relative_axes().iter().copied().collect();
    let absolute_axis: Vec<(AbsoluteAxisCode, u16)> =
        get_all_absolute_axis().iter().copied().collect();

    let mut builder = VirtualDevice::builder()?
        .name("Mudras virtual pointer")
        .with_relative_axes(&relative_axes)?;

    // let abs_info = AbsInfo::new(0, i32::MIN, i32::MAX, 0, 0, 0);
    // for axis in absolute_axis {
    //     builder = builder.with_absolute_axis(&UinputAbsSetup::new(axis.0, abs_info.clone()))?;
    // }

    let device = builder.build();
    match device {
        Ok(device) => Ok(device),
        Err(e) => {
            let message = format!("Failed to create uinput device: \nErr: {:#?}", e);
            let help = "";
            error!("{}", message);
            let err = LibError::builder().msg(&message).help(help).build();
            return Err(err.into());
        }
    }
}
