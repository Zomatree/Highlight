use std::{collections::HashMap, fmt::Debug, ops::Deref};

use revolt_models::v0::Message;

use crate::{
    Error,
    Context as MessageContext,
    commands::{Command, Words},
};

#[derive(Debug)]
pub struct Context<'a, E: From<Error> + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync> {
    pub inner: &'a MessageContext<'a>,
    pub prefix: String,
    pub command: Option<&'a Command<E, S>>,
    pub message: Message,
    pub state: S,
    pub words: Words,
    pub commands: &'a HashMap<String, Command<E, S>>
}

impl<'a, E: From<Error> + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync> Deref
    for Context<'a, E, S>
{
    type Target = MessageContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
