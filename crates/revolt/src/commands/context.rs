use std::{fmt::Debug, ops::Deref};

use revolt_models::v0::Message;

use crate::{
    commands::{handler::Commands, Command, Words}, Context as MessageContext, Error
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
    pub commands: Commands<E, S>,
}

impl<E: From<Error> + Clone + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync> Deref
    for Context<E, S>
{
    type Target = MessageContext;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
