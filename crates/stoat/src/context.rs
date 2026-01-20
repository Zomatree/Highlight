use std::sync::Arc;

use stoat_database::events::server::ClientMessage;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    Error, GlobalCache, HttpClient,
    notifiers::Notifiers,
    websocket::{EventMessage, ProgramMessage},
};

#[derive(Debug, Clone)]
pub struct Events(pub(crate) Arc<UnboundedSender<EventMessage>>);

impl AsRef<Events> for Events {
    fn as_ref(&self) -> &Events {
        self
    }
}

impl Events {
    pub(crate) fn send_message(&self, message: EventMessage) -> Result<(), Error> {
        self.0.send(message).map_err(|_| Error::BrokenChannel)
    }

    pub fn send_event(&self, event: ClientMessage) -> Result<(), Error> {
        self.send_message(EventMessage::Client(event))
    }

    pub fn close(&self) -> Result<(), Error> {
        self.send_message(EventMessage::Program(ProgramMessage::Close))
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub cache: GlobalCache,
    pub http: HttpClient,
    pub notifiers: Notifiers,
    pub events: Events,
}

impl AsRef<GlobalCache> for Context {
    fn as_ref(&self) -> &GlobalCache {
        &self.cache
    }
}

impl AsRef<HttpClient> for Context {
    fn as_ref(&self) -> &HttpClient {
        &self.http
    }
}

impl AsRef<Notifiers> for Context {
    fn as_ref(&self) -> &Notifiers {
        &self.notifiers
    }
}

impl AsRef<Events> for Context {
    fn as_ref(&self) -> &Events {
        &self.events
    }
}
