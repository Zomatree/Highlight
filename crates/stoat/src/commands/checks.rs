use std::{fmt::Debug, sync::Arc};

use async_fn_traits::AsyncFn1;
use async_trait::async_trait;
use stoat_models::v0::Channel;
use stoat_permissions::ChannelPermission;

use crate::{Error, commands::Context};

#[async_trait]
pub trait Check<E, S>: Send + Sync + 'static {
    async fn run(&self, context: Context<E, S>) -> Result<bool, E>;
}

#[async_trait]
impl<E, S, F> Check<E, S> for F
where
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    F: AsyncFn1<Context<E, S>, Output = Result<bool, E>> + Send + Sync + 'static,
    F::OutputFuture: Send + Sync,
{
    async fn run(&self, context: Context<E, S>) -> Result<bool, E> {
        (self)(context).await
    }
}

pub struct HasChannelPermissions(Vec<ChannelPermission>);

impl HasChannelPermissions {
    pub fn new(permissions: Vec<ChannelPermission>) -> Self {
        Self(permissions)
    }
}

#[async_trait]
impl<
    E: From<Error> + Send + Sync + 'static,
    S: Send + Sync + 'static,
> Check<E, S> for HasChannelPermissions
{
    async fn run(&self, context: Context<E, S>) -> Result<bool, E> {
        let permissions = context.get_author_channel_permissions().await;

        for perm in &self.0 {
            if !permissions.has(*perm as u64) {
                return Err(Error::MissingChannelPermission { permissions: *perm }.into());
            };
        }

        Ok(true)
    }
}

pub struct HasServerPermissions(Vec<ChannelPermission>);

impl HasServerPermissions {
    pub fn new(permissions: Vec<ChannelPermission>) -> Self {
        Self(permissions)
    }
}

#[async_trait]
impl<
    E: From<Error> + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Check<E, S> for HasServerPermissions
{
    async fn run(&self, context: Context<E, S>) -> Result<bool, E> {
        let permissions = context.get_author_server_permissions().await;

        for perm in &self.0 {
            if !permissions.has(*perm as u64) {
                return Err(Error::MissingChannelPermission { permissions: *perm }.into());
            };
        }

        Ok(true)
    }
}

pub struct CheckAny<E, S>(pub Arc<Vec<Box<dyn Check<E, S>>>>);

#[async_trait]
impl<
    E: From<Error> + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
> Check<E, S> for CheckAny<E, S>
{
    async fn run(&self, context: Context<E, S>) -> Result<bool, E> {
        for check in self.0.iter() {
            if check.run(context.clone()).await.unwrap_or_default() == true {
                return Ok(true);
            }
        }

        Err(Error::CheckFailure.into())
    }
}

impl<E, S> CheckAny<E, S> {
    pub fn new(checks: Vec<Box<dyn Check<E, S>>>) -> Self {
        Self(Arc::new(checks))
    }
}

pub async fn server_only<
    E: From<Error> + Send + Sync + 'static,
    S: Send + Sync + 'static,
>(
    context: Context<E, S>,
) -> Result<bool, E> {
    match context.get_current_channel().await {
        Some(Channel::TextChannel { .. } | Channel::VoiceChannel { .. }) => Ok(true),
        _ => Err(Error::NotInServer.into()),
    }
}

pub async fn dm_only<
    E: From<Error> + Send + Sync + 'static,
    S: Send + Sync + 'static,
>(
    context: Context<E, S>,
) -> Result<bool, E> {
    match context.get_current_channel().await {
        Some(
            Channel::DirectMessage { .. } | Channel::Group { .. } | Channel::SavedMessages { .. },
        ) => Ok(true),
        _ => Err(Error::NotInDM.into()),
    }
}
