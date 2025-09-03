use std::{
    collections::HashMap, fmt::{self, Debug}, marker::PhantomData, sync::Arc
};

use async_trait::async_trait;
use async_fn_traits::{AsyncFn1, AsyncFn2, AsyncFn3};

use crate::{
    commands::{Context, Converter}, Error
};

#[derive(Clone)]
pub struct Command<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync,
> {
    pub name: String,
    pub handle: Arc<Box<dyn CommandHandle<(), E, S>>>,
    pub children: HashMap<String, Command<E, S>>,
    pub description: Option<String>,
}

impl<E: From<Error> + Clone + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync>
    fmt::Debug for Command<E, S>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("children", &self.children)
            .finish_non_exhaustive()
    }
}

impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Command<E, S>
{
    pub fn new<T: Send + Sync + 'static, I: Into<String>, F: CommandHandle<T, E, S> + Clone>(name: I, handle: F) -> Self {
        let erased = ErasedCommandHandler { handle, _p: PhantomData };

        Self {
            name: name.into(),
            handle: Arc::new(Box::new(erased)),
            children: HashMap::new(),
            description: None,
        }
    }

    pub fn child(mut self, command: Self) -> Self {
        self.children.insert(command.name.clone(), command);

        self
    }

    pub fn description<I: Into<String>>(mut self, description: I) -> Self {
        self.description = Some(description.into());

        self
    }
}

#[async_trait]
pub trait CommandHandle<
    T,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>: Send + Sync + 'static
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E>;
}

#[async_trait]
impl<E, S, F> CommandHandle<(), E, S> for F
where
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    F: AsyncFn1<Context<E, S>, Output = Result<(), E>> + Send + Sync + 'static,
    F::OutputFuture: Send
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
    F::OutputFuture: Send
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
    F::OutputFuture: Send
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        let t1 = T1::from_context(&context).await?;
        let t2 = T2::from_context(&context).await?;

        (self)(context, t1, t2).await
    }
}

struct ErasedCommandHandler<
    T: Send + Sync + 'static,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    H: CommandHandle<T, E, S>
> {
    handle: H,
    _p: PhantomData<(T, E, S)>
}

#[async_trait]
impl<
    T: Send + Sync + 'static,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    H: CommandHandle<T, E, S>
> CommandHandle<(), E, S> for ErasedCommandHandler<T, E, S, H> {
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        self.handle.handle(context).await
    }
}
