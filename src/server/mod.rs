pub mod events;
mod signal;

use self::events::{Event, EventHandler};
use crate::config::Config;

use bon::bon;
use futures::StreamExt;
use tokio::select;
use tokio::task::JoinHandle;

// Globals
use std::sync::{Arc, RwLock};
// Error
use crate::error::{LibError, MudrasError, WrapError};
use miette::Result;
use tracing::{debug, error, info, trace, warn};

#[derive(Debug, Clone)]
pub struct Server {
    config: Arc<RwLock<Config>>,
    events: EventHandler,
    tasks: Vec<Arc<JoinHandle<()>>>,
}

#[bon]
impl Server {
    #[builder]
    pub async fn new(config: Config) -> Result<Self, MudrasError> {
        let tasks = vec![];
        let res = Self {
            config: Arc::new(RwLock::new(config)),
            events: EventHandler::default(),
            tasks,
        };
        Ok(res)
    }
}
impl Server {
    pub async fn run(&mut self) -> Result<(), MudrasError> {
        // Launch tasks
        let rx_task: JoinHandle<()> = tokio::spawn({
            let res = self.clone();
            async move {
                _ = res.handle_events().await;
            }
        });
        let signal_task: JoinHandle<()> = tokio::spawn({
            let res = self.clone();
            async move {
                _ = res.handle_signals().await;
            }
        });
        self.tasks
            .extend(vec![Arc::new(rx_task), Arc::new(signal_task)]);
        let config = self.config.read().unwrap().clone();

        self.listen_keyboard(&config).await?;
        Ok(())
    }
}

impl Server {
    #[tracing::instrument(skip_all)]
    pub async fn handle_events(&self) -> Result<(), MudrasError> {
        let events = self.events.clone();
        let mut receiver = events.sender.subscribe();
        loop {
            if events.sender.receiver_count() == 0 {
                break;
            }
            select! {
                biased;
                Ok(event) = receiver.recv() => {
                    match event {
                        Event::Quit => {}
                        Event::Action => {}

                    }
                }
            }
        }
        Ok(())
    }
}
