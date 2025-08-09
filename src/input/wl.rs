use crate::backend::tty::Tty;
use crate::config::Bind;

// Globals
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::config::Config;
use smithay::backend::input::{
    DeviceCapability, Event, InputBackend, InputEvent, KeyState, KeyboardKeyEvent, Keycode,
};
use smithay::backend::libinput::LibinputInputBackend;
use smithay::backend::udev::{UdevBackend, UdevEvent};
use smithay::input::keyboard::{keysyms, FilterResult, Keysym, Layout, ModifiersState};
use smithay::input::{keyboard, Seat, SeatHandler, SeatState};
use smithay::reexports::calloop::{EventLoop, LoopHandle, LoopSignal};
use smithay::reexports::input::Device;

use smithay::utils::SERIAL_COUNTER;

// Error
use crate::error::{LibError, MudrasError, WrapError};
use log::{debug, error, info, trace, warn};
use miette::{Error, IntoDiagnostic, Result};

pub struct State {
    pub backend: Tty,
    pub config: Rc<RefCell<Config>>,
    pub event_loop: LoopHandle<'static, State>,
    pub stop_signal: LoopSignal,
    pub devices: HashSet<Device>,
    // pub seat: Seat<State>,
}

impl State {
    pub fn new(
        config: Rc<RefCell<Config>>,
        event_loop: LoopHandle<'static, State>,
        stop_signal: LoopSignal,
        devices: HashSet<Device>,
    ) -> Result<Self, MudrasError> {
        let config_ = config.borrow();

        let backend = Tty::new(config.clone(), event_loop.clone()).unwrap();
        let devices = HashSet::new();

        // let mut seat_state = SeatState::new();
        // let mut seat: Seat<State> = seat_state.new_seat("tty");
        // seat.add_keyboard(
        //     XkbConfig {
        //         layout: "us",
        //         ..XkbConfig::default()
        //     },
        //     200,
        //     25,
        // )
        // .unwrap();

        let state = Self {
            config,
            event_loop,
            stop_signal,
            backend,
            devices,
            // seat,
        };
        Ok(state)
    }
    pub fn process_input_event<I: InputBackend<Device = Device> + 'static>(
        &mut self,
        event: InputEvent<I>,
    ) where
        I::Device: 'static, // Needed for downcasting.
    {
        use InputEvent::*;
        match event {
            DeviceAdded { device } => self.on_device_added(device),
            DeviceRemoved { device } => self.on_device_removed(device),
            Keyboard { event } => self.on_keyboard::<I>(event),
            _ => {}
        }
    }
    pub fn process_libinput_event(&mut self, event: &mut InputEvent<LibinputInputBackend>) {
        match event {
            InputEvent::DeviceAdded { device } => {
                if device.has_capability(DeviceCapability::Keyboard.into()) {
                    self.devices.insert(device.clone());
                }
            }
            InputEvent::DeviceRemoved { device } => {
                self.devices.remove(device);
            }
            _ => (),
        }
    }
    fn on_device_added(&mut self, device: Device) {}
    fn on_device_removed(&mut self, device: Device) {}
    fn on_keyboard<I: InputBackend>(&mut self, event: I::KeyboardKeyEvent) {
        let comp_mod = self.backend.mod_key();

        let serial = SERIAL_COUNTER.next_serial();
        let time = Event::time_msec(&event);
        let pressed = event.state() == KeyState::Pressed;

        // let Some(Some(bind)) = self.seat.get_keyboard().unwrap().input(
        //     self,
        //     event.key_code(),
        //     event.state(),
        //     serial,
        //     time,
        //     |this, mods, keysym| {
        //         let bindings = &this.config.borrow().binds;
        //         let key_code = event.key_code();
        //         let modified = keysym.modified_sym();
        //         let raw = keysym.raw_latin_sym_or_raw_current_sym();
        //     },
        // ) else {
        //     return;
        // };

        // if !pressed {
        //     return;
        // }

        // self.handle_bind(bind.clone());
        // self.start_key_repeat(bind);
    }

    fn start_key_repeat(&mut self, bind: Bind) {}
    pub fn handle_bind(&mut self, bind: Bind) {}
    // pub fn do_action(&mut self, action: Action, allow_when_locked: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_create_state() -> Result<()> {
        let config = Config::get()?;
        let mut event_loop = EventLoop::try_new().unwrap();
        let devices = HashSet::new();
        let mut state = State::new(
            config,
            event_loop.handle(),
            event_loop.get_signal(),
            devices,
        )?;

        // Run the compositor.
        event_loop.run(None, &mut state, || {}).unwrap();

        Ok(())
    }
}
