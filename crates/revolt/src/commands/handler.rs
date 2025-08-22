use std::{collections::HashMap, fmt::Debug};
use futures::{future::BoxFuture, FutureExt};
use revolt_models::v0::Message;
use crate::{commands::{Command, CommandEventHandler, Context, Words}, Context as MessageContext, Error};


pub struct CommandHandler<
    H: CommandEventHandler<E, S> + Send + Sync,
    E: From<Error> + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync,
> {
    prefix: Box<
        dyn for<'a> Fn(&'a MessageContext<'_>, &'a Message) -> BoxFuture<'a, Result<Vec<String>, E>>
            + Send
            + Sync,
    >,
    commands: HashMap<String, Command<E, S>>,
    event_handler: H,
    state: S,
}

impl<
    H: CommandEventHandler<E, S> + Send + Sync,
    E: From<Error> + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync,
> CommandHandler<H, E, S>
{
    pub fn new(event_handler: H, state: S) -> Self {
        Self {
            prefix: Box::new(|_, _| panic!("No prefix")),
            commands: HashMap::new(),
            event_handler,
            state,
        }
    }

    pub fn with_static_prefix(mut self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();

        self.prefix = Box::new(move |_, _| {
            let prefix = prefix.clone();
            async { Ok(vec![prefix]) }.boxed()
        });

        self
    }

    pub fn with_static_prefixes(mut self, prefixes: Vec<String>) -> Self {
        self.prefix = Box::new(move |_, _| {
            let prefixes = prefixes.clone();
            async { Ok(prefixes) }.boxed()
        });

        self
    }

    pub fn with_prefix<
        Fut: Future<Output = Result<Vec<String>, E>> + Send + Sync + 'static,
        F: for<'a> Fn(&'a MessageContext<'_>, &'a Message) -> Fut + Send + Sync + 'static,
    >(
        mut self,
        f: F,
    ) -> Self {
        self.prefix = Box::new(move |a, b| f(a, b).boxed());

        self
    }

    pub fn register(mut self, commands: Vec<Command<E, S>>) -> Self {
        for command in commands {
            self.commands.insert(command.name.clone(), command);
        }

        self
    }

    pub fn find_command_from_words<'a>(&'a self, current_command: Option<&'a Command<E, S>>, words: &mut Words) -> Option<&'a Command<E, S>> {
        let next_word = words.current()?;
        println!("{next_word:?}");

        if let Some(command) = current_command.and_then(|command| command.children.get(&next_word)).or_else(|| self.commands.get(&next_word)) {
            println!("{command:?}");
            words.advance();

            if !command.children.is_empty() {
                let subcommand = self.find_command_from_words(Some(command), words);
                println!("{subcommand:?}");

                match subcommand {
                    Some(sub) => Some(sub),
                    None => {
                        words.undo();

                        Some(command)
                    }
                }
            } else {
                Some(command)
            }
        } else {
            None
        }
    }

    pub async fn process_commands(
        &self,
        context: &MessageContext<'_>,
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

        let mut words = Words::new(rest);
        println!("{words:?}");

        let mut context = Context {
            inner: context,
            prefix: prefix,
            command: self.find_command_from_words(None, &mut words),
            message: message.clone(),
            state: self.state.clone(),
            words,
        };

        println!("{:?}", context.command);

        if context.command.is_none() {
            if let Err(e) = self.event_handler.no_command(&mut context).await {
                self.event_handler.error(&mut context, e).await.unwrap();
            };

            return Ok(())
        }

        if let Err(e) = self.event_handler.command(&mut context).await {
            self.event_handler.error(&mut context, e).await.unwrap();
        };

        if let Some(handle) = context.command.map(|c| c.handle) {
            if let Err(e) = handle(&mut context).await {
                self.event_handler.error(&mut context, e).await.unwrap();
            };
        }

        Ok(())
    }
}
