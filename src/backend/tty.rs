use crate::config::Config;
use crate::input::wl::State;

// Globals
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use smithay::reexports::calloop::{Dispatcher, LoopHandle, RegistrationToken};

use smithay::backend::session::libseat::LibSeatSession;
use smithay::input::{Seat, SeatHandler, SeatState};

use smithay::backend::input::{DeviceCapability, InputBackend};
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::reexports::input::Libinput;
use smithay::wayland::input_method::{InputMethodManagerState, InputMethodSeat};

use smithay::backend::udev::{UdevBackend, UdevEvent};

// Error
use crate::error::{LibError, MudrasError, WrapError};
use log::{debug, error, info, trace, warn};
use miette::{miette, Error, IntoDiagnostic, Result};

pub struct Tty {
    config: Rc<RefCell<Config>>,
    session: LibSeatSession,
    udev_dispatcher: Dispatcher<'static, UdevBackend, State>,
    libinput: Libinput,
    // devices: HashMap<DrmNode, OutputDevice>,
}
impl Tty {
    pub fn new(
        config: Rc<RefCell<Config>>,
        event_loop: LoopHandle<'static, State>,
    ) -> Result<Self> {
        let (session, notifier) = LibSeatSession::new().context(
            "Error creating a session. This might mean that you're trying to run mudras on a TTY \
             that is already busy, for example if you're running this inside tmux that had been \
             originally started on a different TTY",
        )?;
        let seat_name = session.seat();

        let udev_backend =
            UdevBackend::new(session.seat()).context("error creating a udev backend")?;
        let udev_dispatcher = Dispatcher::new(udev_backend, move |event, _, state: &mut State| {
            state.backend.tty().on_udev_event(&mut state, event);
        });
        event_loop
            .register_dispatcher(udev_dispatcher.clone())
            .unwrap();

        let mut libinput = Libinput::new_with_udev(LibinputSessionInterface::from(session.clone()));
        libinput
            .udev_assign_seat(&seat_name)
            .map_err(|()| miette!("error assigning the seat to libinput"))?;

        let input_backend = LibinputInputBackend::new(libinput.clone());
        event_loop
            .insert_source(input_backend, |mut event, _, state| {
                state.process_libinput_event(&mut event);
                state.process_input_event(event);
            })
            .unwrap();

        event_loop
            .insert_source(notifier, move |event, _, state| {
                state.backend.tty().on_session_event(&mut state, event);
            })
            .unwrap();

        Ok(Self {
            config,
            session,
            udev_dispatcher,
            libinput,
            // devices: HashMap::new(),
        })
    }
}
