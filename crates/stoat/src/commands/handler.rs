use crate::{
    Context as MessageContext, Error,
    commands::{Command, CommandEventHandler, Context, Words},
};
use async_recursion::async_recursion;
use futures::{FutureExt, future::BoxFuture};
use stoat_models::v0::Message;
use state::TypeMap;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CommandHandler<H: CommandEventHandler + Clone + Send + Sync + 'static> {
    prefix: Arc<
        Box<
            dyn for<'a> Fn(
                    &'a MessageContext,
                    &'a Message,
                ) -> BoxFuture<'a, Result<Vec<String>, H::Error>>
                + Send
                + Sync,
        >,
    >,
    commands: Commands<H::Error, H::State>,
    event_handler: H,
    state: H::State,
}

impl<
    H: CommandEventHandler<State = S, Error = E> + Clone + Send + Sync,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> CommandHandler<H>
{
    pub fn new(event_handler: H, state: S) -> Self {
        Self {
            prefix: Arc::new(Box::new(|_, _| async move { Ok(vec![]) }.boxed())),
            commands: Commands::new(),
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
            self.commands.try_register(command)
        }

        self
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

        let cmd_context = Context {
            inner: context,
            prefix: prefix,
            command: self.commands.find_command_from_words(None, &words).await,
            message: message.clone(),
            state: self.state.clone(),
            words,
            commands: self.commands.clone(),
            local_state: Arc::new(<TypeMap![Send + Sync]>::new()),
        };

        if cmd_context.command.is_none() {
            if let Err(e) = self.event_handler.no_command(cmd_context.clone()).await {
                self.event_handler
                    .error(cmd_context.clone(), e)
                    .await
                    .unwrap();
            };

            return Ok(());
        }

        if let Err(e) = self.event_handler.command(cmd_context.clone()).await {
            self.event_handler
                .error(cmd_context.clone(), e)
                .await
                .unwrap();
        };

        if let Some(command) = cmd_context.command.as_ref() {
            if let Err(e) = command.can_run(cmd_context.clone()).await {
                self.event_handler
                    .error(cmd_context.clone(), e)
                    .await
                    .unwrap();
            } else {
                if let Err(e) = command.handle.handle(cmd_context.clone()).await {
                    self.event_handler
                        .error(cmd_context.clone(), e)
                        .await
                        .unwrap();
                };
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Commands<E, S> {
    mapping: Arc<RwLock<HashMap<String, Command<E, S>>>>,
}

impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Commands<E, S>
{
    pub fn new() -> Self {
        Self {
            mapping: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn try_register(&self, command: Command<E, S>) {
        let mut mapping = self.mapping.try_write().unwrap();

        mapping.insert(command.name.clone(), command.clone());

        for alias in command.aliases.clone() {
            mapping.insert(alias, command.clone());
        }
    }

    pub async fn register(&self, command: Command<E, S>) {
        let mut mapping = self.mapping.write().await;

        mapping.insert(command.name.clone(), command.clone());

        for alias in command.aliases.clone() {
            mapping.insert(alias, command.clone());
        }
    }

    #[async_recursion]
    pub async fn find_command_from_words(
        &self,
        current_command: Option<&Command<E, S>>,
        words: &Words,
    ) -> Option<Command<E, S>> {
        let next_word = words.current()?;

        let commands = self.mapping.read().await;

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

    pub async fn get_command_from_slice(&self, words: &[String]) -> Option<Command<E, S>> {
        let mapping = self.mapping.read().await;

        let mut current_command: Option<Command<E, S>> = None;

        for word in words {
            if let Some(command) = current_command
                .as_ref()
                .and_then(|command| command.get_command(word))
                .or_else(|| mapping.get(word).cloned())
            {
                current_command = Some(command)
            } else {
                break;
            }
        }

        return current_command;
    }

    pub async fn get_command(&self, name: &str) -> Option<Command<E, S>> {
        self.mapping.read().await.get(name).cloned()
    }

    pub async fn get_commands(&self) -> Vec<Command<E, S>> {
        self.mapping
            .read()
            .await
            .clone()
            .into_iter()
            .filter(|(name, command)| name == &command.name)
            .map(|(_, command)| command)
            .collect()
    }

    pub async fn get_command_parents(&self, command: &Command<E, S>) -> Vec<Command<E, S>> {
        let mut parents: Vec<Command<E, S>> = Vec::new();

        for parent in &command.parents {
            if let Some(last_parent) = parents.last() {
                let child = last_parent.get_command(parent).unwrap();
                parents.push(child);
            } else {
                parents.push(self.get_command(parent).await.unwrap());
            }
        };

        parents
    }
}
