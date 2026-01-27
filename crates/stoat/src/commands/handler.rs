use crate::{
    Context as MessageContext, Error,
    commands::{
        Check, Command, CommandEventHandler, Context, DefaultHelpCommand, HelpCommand, Words,
        help_command,
    },
};
use state::TypeMap;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};
use stoat_models::v0::Message;

#[derive(Clone)]
pub struct CommandHandler<H: CommandEventHandler + Clone + Send + Sync + 'static> {
    commands: Commands<H::Error, H::State>,
    checks: Vec<Arc<dyn Check<H::Error, H::State>>>,
    event_handler: H,
    state: H::State,
    help_command: Arc<dyn HelpCommand<H::Error, H::State>>,
}

impl<
    H: CommandEventHandler<State = S, Error = E> + Clone + Send + Sync,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> CommandHandler<H>
{
    pub fn new(event_handler: H, state: S) -> Self {
        let commands = Commands::new();
        commands.register(help_command());

        Self {
            commands,
            checks: Vec::new(),
            event_handler,
            state,
            help_command: Arc::new(DefaultHelpCommand),
        }
    }

    pub fn register(self, commands: Vec<Command<E, S>>) -> Self {
        for command in commands {
            self.commands.register(command)
        }

        self
    }

    pub fn help_command<HC: HelpCommand<E, S> + 'static>(
        mut self,
        help_command: Option<HC>,
    ) -> Self {
        if let Some(help_command) = help_command {
            self.help_command = Arc::new(help_command);
        } else if let Some(help_command) = self.commands.get_command("help") {
            self.commands.unregister(help_command);
        }

        self
    }

    pub fn check<C: Check<E, S>>(mut self, check: C) -> Self {
        self.checks.push(Arc::new(check));

        self
    }

    pub async fn can_run(&self, context: Context<E, S>) -> Result<bool, E> {
        for check in &self.checks {
            if check.run(context.clone()).await? == false {
                return Err(Error::CheckFailure.into());
            }
        }

        if let Some(command) = &context.command {
            return command.can_run(context.clone()).await;
        }

        Ok(true)
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

        if message.user.as_ref().unwrap().bot.is_some() {
            return Ok(());
        };

        let mut cmd_context = Context {
            inner: context,
            prefix: None,
            command: None,
            message: message.clone(),
            state: self.state.clone(),
            words: Words::new(message_content),
            commands: self.commands.clone(),
            help_command: self.help_command.clone(),
            local_state: Arc::new(<TypeMap![Send + Sync]>::new()),
        };

        let prefixes = self.event_handler.get_prefix(cmd_context.clone()).await?;

        let Some(prefix) = prefixes
            .into_iter()
            .filter(|prefix| message_content.starts_with(prefix))
            .next()
        else {
            // doesnt start with prefix
            return Ok(());
        };

        let rest = &message_content[prefix.len()..];

        cmd_context.words = Words::new(rest);
        cmd_context.command = self
            .commands
            .find_command_from_words(None, &cmd_context.words);
        cmd_context.prefix = Some(prefix);

        if cmd_context.command.is_none() {
            if let Err(e) = self.event_handler.no_command(cmd_context.clone()).await {
                self.event_handler.error(cmd_context.clone(), e).await?;
            };

            return Ok(());
        }

        if let Err(e) = self.event_handler.command(cmd_context.clone()).await {
            self.event_handler.error(cmd_context.clone(), e).await?;
        };

        if let Some(command) = cmd_context.command.as_ref() {
            if let Err(e) = self.can_run(cmd_context.clone()).await {
                self.event_handler.error(cmd_context.clone(), e).await?;
            } else {
                if let Err(e) = command.handle.handle(cmd_context.clone()).await {
                    self.event_handler.error(cmd_context.clone(), e).await?;
                };

                if let Err(e) = self.event_handler.after_command(cmd_context.clone()).await {
                    self.event_handler.error(cmd_context.clone(), e).await?;
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

    pub fn register(&self, command: Command<E, S>) {
        let mut mapping = self.mapping.write().unwrap();

        mapping.insert(command.name.clone(), command.clone());

        for alias in command.aliases.clone() {
            mapping.insert(alias, command.clone());
        }
    }

    pub fn unregister(&self, command: Command<E, S>) {
        let mut mapping = self.mapping.write().unwrap();

        mapping.remove(&command.name);

        for alias in command.aliases.clone() {
            mapping.remove(&alias);
        }
    }

    pub fn find_command_from_words(
        &self,
        current_command: Option<&Command<E, S>>,
        words: &Words,
    ) -> Option<Command<E, S>> {
        let next_word = words.current()?;

        let commands = self.mapping.read().unwrap();

        if let Some(command) = current_command
            .and_then(|command| command.children.get(&next_word))
            .or_else(|| commands.get(&next_word))
        {
            words.advance();

            if !command.children.is_empty() {
                let subcommand = self.find_command_from_words(Some(command), words);

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

    pub fn get_command_from_slice(&self, words: &[String]) -> Option<Command<E, S>> {
        let mapping = self.mapping.read().unwrap();

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

    pub fn get_command(&self, name: &str) -> Option<Command<E, S>> {
        self.mapping.read().unwrap().get(name).cloned()
    }

    pub fn get_commands(&self) -> Vec<Command<E, S>> {
        self.mapping
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(name, command)| name == &command.name)
            .map(|(_, command)| command)
            .collect()
    }

    pub fn get_command_parents(&self, command: &Command<E, S>) -> Vec<Command<E, S>> {
        let mut parents: Vec<Command<E, S>> = Vec::new();

        for parent in &command.parents {
            if let Some(last_parent) = parents.last() {
                let child = last_parent.get_command(parent).unwrap();
                parents.push(child);
            } else {
                parents.push(self.get_command(parent).unwrap());
            }
        }

        parents
    }
}
