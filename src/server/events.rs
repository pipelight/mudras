use tokio::sync::broadcast::{self, Receiver, Sender};

// Error
use crate::error::{LibError, MudrasError, WrapError};
use miette::Result;
use tracing::{debug, error, info, trace, warn};

#[derive(Debug, Clone)]
pub enum Event {
    Quit,
    Action,
}

/// Terminal event handler.
#[derive(Debug, Clone)]
pub struct EventHandler {
    /// Event sender channel.
    pub sender: Sender<Event>,
}

impl Default for EventHandler {
    fn default() -> Self {
        let (sender, _receiver) = broadcast::channel(10);
        Self { sender }
    }
}

impl EventHandler {
    /// Queue an event to be sent to the event receiver.
    /// This is useful for sending events to the event handler which will be processed by
    /// the next iteration of the application's event loop.
    #[tracing::instrument(skip(self))]
    pub fn send(&self, event: Event) -> Result<(), MudrasError> {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        match self.sender.send(event) {
            Ok(_size) => Ok(()),
            Err(_e) => {
                let message = "Coundn't send event.";
                let help = "";
                Err(LibError::builder().msg(message).help(help).build().into())
            }
        }
    }
}
