use std::{collections::HashMap, fmt::Debug, ops::Deref};

use crate::Error;
use async_trait::async_trait;
use futures::{FutureExt, future::BoxFuture};
use revolt_models::v0::Message;

use crate::Context as MessageContext;

pub type CommandReturn<'a, E> = BoxFuture<'a, Result<(), E>>;

pub struct Context<'a, E: Debug + Send + 'static, S: Clone + Send + Sync> {
    pub inner: &'a MessageContext<'a>,
    pub prefix: String,
    pub command: Option<&'a Command<E, S>>,
    pub message: Message,
    pub state: S,
    pub words: Words,
}

impl<'a, E: Debug + Send + 'static, S: Clone + Send + Sync> Deref for Context<'a, E, S> {
    type Target = MessageContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Command<E: Debug + Send + 'static, S: Clone + Send + Sync> {
    pub name: String,
    pub handle: for<'a> fn(&'a mut Context<'_, E, S>) -> CommandReturn<'a, E>,
}

#[async_trait]
pub trait Converter<E: Debug + Send + Sync, S: Clone + Send + Sync>: Sized {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E>;
}

#[async_trait]
impl<E: From<Error> + Debug + Send + Sync, S: Clone + Send + Sync> Converter<E, S> for u32 {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E> {
        input
            .parse::<u32>()
            .map_err(|e| Error::ConverterError(e.to_string()))
            .map_err(|e| e.into())
    }
}

#[async_trait]
impl<E: From<Error> + Debug + Send + Sync, S: Clone + Send + Sync> Converter<E, S> for String {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E> {
        Ok(input)
    }
}

pub struct ConsumeRest(pub String);

#[async_trait]
impl<E: From<Error> + Debug + Send + Sync, S: Clone + Send + Sync> Converter<E, S> for ConsumeRest {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E> {
        let mut output = input;

        let rest = context.words.rest().join(" ");

        if !rest.is_empty() {
            output.push(' ');
            output.push_str(&rest);
        };

        Ok(ConsumeRest(output))
    }
}

#[async_trait]
#[allow(unused)]
pub trait CommandEventHandler<E: Debug + Send + 'static, S: Clone + Send + Sync> {
    async fn command(&self, context: &mut Context<'_, E, S>) -> Result<(), E> {
        Ok(())
    }

    async fn no_command(&self, context: &mut Context<'_, E, S>) -> Result<(), E> {
        Ok(())
    }

    async fn error(&self, context: &mut Context<'_, E, S>, error: E) {
        println!("Error: {error:?}")
    }
}

#[derive(Debug, Clone)]
pub struct Words {
    values: Vec<String>,
    pos: usize,
}

impl Words {
    fn new(input: &str) -> Self {
        Self {
            values: input.split(' ').map(|v| v.to_string()).collect(),
            pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<String> {
        let value = self.values.get(self.pos).cloned();
        self.pos += 1;

        value
    }

    pub fn current(&self) -> Option<String> {
        self.values.get(self.pos).cloned()
    }

    pub fn rest(&self) -> Vec<String> {
        self.values.iter().skip(self.pos).cloned().collect()
    }
}

pub struct CommandHandler<
    H: CommandEventHandler<E, S> + Send + Sync,
    E: Debug + Send + Sync + 'static,
    S: Clone + Send + Sync,
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
    E: Debug + Send + Sync + 'static,
    S: Clone + Send + Sync,
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

        let Some(command_name) = words.next() else {
            // no words after
            return Ok(());
        };

        let mut context = Context {
            inner: context,
            prefix: prefix,
            command: None,
            message: message.clone(),
            state: self.state.clone(),
            words,
        };

        let Some(command) = self.commands.get(&command_name) else {
            if let Err(e) = self.event_handler.no_command(&mut context).await {
                self.event_handler.error(&mut context, e).await;
            }

            return Ok(());
        };

        context.command = Some(command);

        if let Err(e) = self.event_handler.command(&mut context).await {
            self.event_handler.error(&mut context, e).await;
        };

        if let Err(e) = (command.handle)(&mut context).await {
            self.event_handler.error(&mut context, e).await;
        };

        Ok(())
    }
}
