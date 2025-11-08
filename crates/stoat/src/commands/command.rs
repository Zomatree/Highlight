use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
    sync::Arc,
};

use async_fn_traits::{AsyncFn1, AsyncFn2, AsyncFn3, AsyncFn4};
use async_trait::async_trait;

use crate::{
    Error,
    commands::{Context, Converter, checks::Check},
};

#[derive(Clone)]
pub struct Command<E, S> {
    pub name: String,
    pub handle: Arc<Box<dyn CommandHandle<(), E, S>>>,
    pub children: HashMap<String, Command<E, S>>,
    pub checks: Vec<Arc<dyn Check<E, S>>>,
    pub aliases: Vec<String>,
    pub description: Option<String>,
    pub signature: Option<String>,
    pub parents: Vec<String>,
}

impl<E, S> fmt::Debug for Command<E, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("children", &self.children)
            .field("aliases", &self.aliases)
            .field("signature", &self.signature)
            .finish_non_exhaustive()
    }
}

impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Command<E, S>
{
    pub fn new<T: Send + Sync + 'static, I: Into<String>, F: CommandHandle<T, E, S> + Clone>(
        name: I,
        handle: F,
    ) -> Self {
        let erased = ErasedCommandHandler {
            handle,
            _p: PhantomData,
        };

        Self {
            name: name.into(),
            handle: Arc::new(Box::new(erased)),
            children: HashMap::new(),
            checks: Vec::new(),
            aliases: Vec::new(),
            description: None,
            signature: None,
            parents: Vec::new(),
        }
    }

    pub fn child(mut self, mut command: Self) -> Self {
        command.parents = self.parents.clone();
        command.parents.push(self.name.clone());

        self.children.insert(command.name.clone(), command.clone());

        for alias in command.aliases.clone() {
            self.children.insert(alias, command.clone());
        }

        self
    }

    pub fn description<I: Into<String>>(mut self, description: I) -> Self {
        self.description = Some(description.into());

        self
    }

    pub fn signature<I: Into<String>>(mut self, signature: I) -> Self {
        self.signature = Some(signature.into());

        self
    }

    pub fn check<C: Check<E, S>>(mut self, check: C) -> Self {
        self.checks.push(Arc::new(check));

        self
    }

    pub fn alias<I: Into<String>>(mut self, alias: I) -> Self {
        self.aliases.push(alias.into());

        self
    }

    pub fn children(&self) -> Vec<Command<E, S>> {
        self.children
            .clone()
            .into_iter()
            .filter(|(name, command)| name == &command.name)
            .map(|(_, command)| command)
            .collect()
    }

    pub fn get_command(&self, name: &str) -> Option<Command<E, S>> {
        self.children.get(name).cloned()
    }

    pub async fn can_run(&self, context: Context<E, S>) -> Result<bool, E> {
        for check in &self.checks {
            if check.run(context.clone()).await? == false {
                return Err(Error::CheckFailure.into());
            }
        }

        Ok(true)
    }
}

#[async_trait]
pub trait CommandHandle<T, E, S>: Send + Sync + 'static {
    async fn handle(&self, context: Context<E, S>) -> Result<(), E>;
}

#[async_trait]
impl<E, S, F> CommandHandle<(), E, S> for F
where
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    F: AsyncFn1<Context<E, S>, Output = Result<(), E>> + Send + Sync + 'static,
    F::OutputFuture: Send,
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        (self)(context).await
    }
}

#[async_trait]
impl<T1, E, S, F> CommandHandle<(T1,), E, S> for F
where
    T1: Converter<E, S> + Send,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    F: AsyncFn2<Context<E, S>, T1, Output = Result<(), E>> + Send + Sync + 'static,
    F::OutputFuture: Send,
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        let t1 = T1::from_context(&context).await?;
        (self)(context, t1).await
    }
}

#[async_trait]
impl<T1, T2, E, S, F> CommandHandle<(T1, T2), E, S> for F
where
    T1: Converter<E, S> + Send,
    T2: Converter<E, S> + Send,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    F: AsyncFn3<Context<E, S>, T1, T2, Output = Result<(), E>> + Send + Sync + 'static,
    F::OutputFuture: Send,
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        let t1 = T1::from_context(&context).await?;
        let t2 = T2::from_context(&context).await?;

        (self)(context, t1, t2).await
    }
}

#[async_trait]
impl<T1, T2, T3, E, S, F> CommandHandle<(T1, T2, T3), E, S> for F
where
    T1: Converter<E, S> + Send,
    T2: Converter<E, S> + Send,
    T3: Converter<E, S> + Send,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    F: AsyncFn4<Context<E, S>, T1, T2, T3, Output = Result<(), E>> + Send + Sync + 'static,
    F::OutputFuture: Send,
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        let t1 = T1::from_context(&context).await?;
        let t2 = T2::from_context(&context).await?;
        let t3 = T3::from_context(&context).await?;

        (self)(context, t1, t2, t3).await
    }
}

struct ErasedCommandHandler<
    T: Send + Sync + 'static,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    H: CommandHandle<T, E, S>,
> {
    handle: H,
    _p: PhantomData<(T, E, S)>,
}

#[async_trait]
impl<
    T: Send + Sync + 'static,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    H: CommandHandle<T, E, S>,
> CommandHandle<(), E, S> for ErasedCommandHandler<T, E, S, H>
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        self.handle.handle(context).await
    }
}
