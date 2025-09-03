use std::{collections::HashMap, fmt::Debug, ops::Deref, sync::Arc};

use revolt_models::v0::Message;
use tokio::sync::RwLock;

use crate::{
    Context as MessageContext, Error,
    commands::{Command, Words},
};

#[derive(Debug, Clone)]
pub struct Context<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> {
    pub inner: MessageContext,
    pub prefix: String,
    pub command: Option<Command<E, S>>,
    pub message: Message,
    pub state: S,
    pub words: Words,
    pub commands: Arc<RwLock<HashMap<String, Command<E, S>>>>,
}

impl<E: From<Error> + Clone + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync> Deref
    for Context<E, S>
{
    type Target = MessageContext;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
