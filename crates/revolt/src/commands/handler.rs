use crate::{
    Context as MessageContext, Error,
    commands::{Command, CommandEventHandler, Context, Words},
};
use async_recursion::async_recursion;
use futures::{FutureExt, future::BoxFuture};
use revolt_models::v0::Message;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CommandHandler<
    H: CommandEventHandler<E, S> + Clone + Send + Sync + 'static,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> {
    prefix: Arc<
        Box<
            dyn for<'a> Fn(&'a MessageContext, &'a Message) -> BoxFuture<'a, Result<Vec<String>, E>>
                + Send
                + Sync,
        >,
    >,
    commands: Arc<RwLock<HashMap<String, Command<E, S>>>>,
    event_handler: H,
    state: S,
}

impl<
    H: CommandEventHandler<E, S> + Clone + Send + Sync,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> CommandHandler<H, E, S>
{
    pub fn new(event_handler: H, state: S) -> Self {
        Self {
            prefix: Arc::new(Box::new(|_, _| panic!("No prefix"))),
            commands: Arc::new(RwLock::new(HashMap::new())),
            event_handler,
            state,
        }
    }

    pub fn with_static_prefix(mut self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();

        self.prefix = Arc::new(Box::new(move |_, _| {
            let prefix = prefix.clone();
            async { Ok(vec![prefix]) }.boxed()
        }));

        self
    }

    pub fn with_static_prefixes(mut self, prefixes: Vec<String>) -> Self {
        self.prefix = Arc::new(Box::new(move |_, _| {
            let prefixes = prefixes.clone();
            async { Ok(prefixes) }.boxed()
        }));

        self
    }

    pub fn with_prefix<
        Fut: Future<Output = Result<Vec<String>, E>> + Send + Sync + 'static,
        F: for<'a> Fn(&'a MessageContext, &'a Message) -> Fut + Send + Sync + 'static,
    >(
        mut self,
        f: F,
    ) -> Self {
        self.prefix = Arc::new(Box::new(move |a, b| f(a, b).boxed()));

        self
    }

    pub fn register(self, commands: Vec<Command<E, S>>) -> Self {
        for command in commands {
            self.commands
                .try_write()
                .unwrap()
                .insert(command.name.clone(), command);
        }

        self
    }

    #[async_recursion]
    pub async fn find_command_from_words(
        &self,
        current_command: Option<&Command<E, S>>,
        words: &Words,
    ) -> Option<Command<E, S>> {
        let next_word = words.current()?;

        let commands = self.commands.read().await;

        if let Some(command) = current_command
            .and_then(|command| command.children.get(&next_word))
            .or_else(|| commands.get(&next_word))
        {
            words.advance();

            if !command.children.is_empty() {
                let subcommand = self.find_command_from_words(Some(command), words).await;

                match subcommand {
                    Some(sub) => Some(sub),
                    None => {
                        words.undo();

                        Some(command.clone())
                    }
                }
            } else {
                Some(command.clone())
            }
        } else {
            None
        }
    }

    pub async fn process_commands(
        &self,
        context: MessageContext,
        message: Message,
    ) -> Result<(), E> {
        let Some(message_content) = message.content.as_deref() else {
            // no content
            return Ok(());
        };

        let prefixes = (self.prefix)(&context, &message).await?;

        let Some(prefix) = prefixes
            .into_iter()
            .filter(|prefix| message_content.starts_with(prefix))
            .next()
        else {
            // doesnt start with prefix
            return Ok(());
        };

        let rest = &message_content[prefix.len()..];

        let words = Words::new(rest);

        let cmd_context = Context::<E, S> {
            inner: context,
            prefix: prefix,
            command: self.find_command_from_words(None, &words).await,
            message: message.clone(),
            state: self.state.clone(),
            words,
            commands: self.commands.clone(),
        };

        if cmd_context.command.is_none() {
            if let Err(e) = self.event_handler.no_command(cmd_context.clone()).await {
                self.event_handler.error(cmd_context.clone(), e).await.unwrap();
            };

            return Ok(());
        }

        if let Err(e) = self.event_handler.command(cmd_context.clone()).await {
            self.event_handler.error(cmd_context.clone(), e).await.unwrap();
        };

        if let Some(handle) = cmd_context.command.as_ref().map(|c| c.handle.clone()) {
            if let Err(e) = handle.handle(cmd_context.clone()).await {
                self.event_handler.error(cmd_context.clone(), e).await.unwrap();
            };
        }

        Ok(())
    }
}
