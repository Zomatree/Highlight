use std::sync::Arc;

use stoat_database::events::server::ClientMessage;
use tokio::sync::mpsc::UnboundedSender;

use crate::{Error, GlobalCache, HttpClient, notifiers::Notifiers};

#[derive(Debug, Clone)]
pub struct Context {
    pub cache: GlobalCache,
    pub http: HttpClient,
    pub notifiers: Notifiers,
    pub(crate) events: Arc<UnboundedSender<ClientMessage>>,
}

impl Context {
    pub fn send_event(&self, event: ClientMessage) -> Result<(), Error> {
        self.events.send(event).map_err(|_| Error::BrokenChannel)
    }
}