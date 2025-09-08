use std::sync::Arc;

use revolt_database::events::server::ClientMessage;
use tokio::sync::mpsc::UnboundedSender;

use crate::{GlobalCache, HttpClient, notifiers::Notifiers};

#[derive(Debug, Clone)]
pub struct Context {
    pub cache: GlobalCache,
    pub http: HttpClient,
    pub notifiers: Notifiers,
    pub events: Arc<UnboundedSender<ClientMessage>>,
}
