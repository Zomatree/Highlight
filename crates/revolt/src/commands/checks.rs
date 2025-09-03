use std::{borrow::Cow, fmt::Debug, sync::Arc};

use async_fn_traits::AsyncFn1;
use async_trait::async_trait;
use revolt_models::v0::Channel;
use revolt_permissions::{calculate_channel_permissions, ChannelPermission};

use crate::{commands::Context, permissions::user_permissions_query, Error};

#[async_trait]
pub trait Check<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>: Send + Sync + 'static {
    async fn run(&self, context: Context<E, S>) -> Result<bool, E>;
}

#[async_trait]
impl<E, S, F> Check<E, S> for F where
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
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Check<E, S> for HasChannelPermissions {
    async fn run(&self, context: Context<E, S>) -> Result<bool, E> {
        let permissions = {
            let user_id = &context.message.author;
            let channel_id = &context.message.channel;

            let mut cache = context.cache.write().await;

            let channel = cache.channels.get(channel_id).ok_or(Error::CheckFailure)?.clone();
            let server_id = match channel {
                Channel::TextChannel { ref server, .. } | Channel::VoiceChannel { ref server, .. } => Some(server),
                _ => None
            };

            let server = server_id.and_then(|id| cache.servers.get(id)).cloned();

            let user = if let Some(user) = cache.users.get(user_id) {
                user.clone()
            } else if let Ok(user) = context.http.fetch_user(user_id).await {
                cache.users.insert(user.id.clone(), user.clone());

                user.clone()
            } else {
                return Err(Error::CheckFailure.into())
            };

            let member = if let Some(server_id) = server_id {
                if let Some(member) = cache.members.get(server_id).as_ref().unwrap().get(user_id) {
                    Some(member.clone())
                } else if let Ok(member) = context.http.fetch_member(&server_id, &user_id).await {
                    cache
                        .members
                        .get_mut(server_id)
                        .unwrap()
                        .insert(user_id.clone(), member.clone());

                    Some(member)
                } else {
                    // in server but member cannot be found even after attempting to fetch member
                    return Err(Error::CheckFailure.into())
                }
            } else {
                None
            };

            let mut query =
                user_permissions_query(&mut cache, context.http.clone(), Cow::Owned(user))
                    .channel(Cow::Owned(channel));

            if let Some(server) = server {
                query = query.server(Cow::Owned(server));
            };

            if let Some(member) = member {
                query = query.member(Cow::Owned(member));
            };

            calculate_channel_permissions(&mut query).await
        };

        for perm in &self.0 {
            if !permissions.has(*perm as u64) {
                return Err(Error::MissingChannelPermission { permissions: *perm }.into());
            };
        };

        Ok(true)
    }
}

pub struct CheckAny<E, S>(pub Arc<Vec<Box<dyn Check<E, S>>>>);

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Check<E, S> for CheckAny<E, S> {
    async fn run(&self, context: Context<E, S>) -> Result<bool, E> {
        for check in self.0.iter() {
            if check.run(context.clone()).await.unwrap_or_default() == true {
                return Ok(true)
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