use std::{borrow::Cow, fmt::Debug, ops::Deref, sync::Arc};

use state::TypeMap;
use stoat_models::v0::{Channel, Member, Message, Server, User};
use stoat_permissions::{
    PermissionValue, calculate_channel_permissions, calculate_server_permissions,
};

use crate::{
    Context as MessageContext, Error, GlobalCache, HttpClient,
    commands::{Command, Words, handler::Commands},
    context::Events,
    notifiers::Notifiers,
    permissions::user_permissions_query,
};

type SendSyncMap = TypeMap![Send + Sync];

#[derive(Debug, Clone)]
pub struct Context<E, S> {
    pub(crate) inner: MessageContext,
    pub prefix: String,
    pub command: Option<Command<E, S>>,
    pub message: Message,
    pub state: S,
    pub words: Words,
    pub commands: Commands<E, S>,
    pub(crate) local_state: Arc<SendSyncMap>,
}

impl<E, S> Context<E, S> {
    pub fn local_cache<F: FnOnce() -> T, T: Send + Sync + 'static>(&self, f: F) -> &T {
        self.local_state.try_get().unwrap_or_else(|| {
            self.local_state.set(f());
            self.local_state.get()
        })
    }

    pub async fn local_cache_async<Fut: Future<Output = T>, T: Send + Sync + 'static>(
        &self,
        fut: Fut,
    ) -> &T {
        match self.local_state.try_get() {
            Some(s) => s,
            None => {
                self.local_state.set(fut.await);
                self.local_state.get()
            }
        }
    }

    pub fn get_current_channel(&self) -> Result<Channel, Error> {
        self.local_cache(|| {
            struct CurrentChannel(Result<Channel, Error>);

            CurrentChannel(
                self.cache
                    .get_channel(&self.message.channel)
                    .ok_or(Error::InternalError),
            )
        })
        .0
        .clone()
    }

    pub fn get_current_server(&self) -> Result<Server, Error> {
        self.local_cache(|| {
            struct CurrentServer(Result<Server, Error>);

            CurrentServer(
                if let Ok(Channel::TextChannel { server, .. }) = self.get_current_channel() {
                    self.cache.get_server(&server).ok_or(Error::InternalError)
                } else {
                    Err(Error::InternalError)
                },
            )
        })
        .0
        .clone()
    }

    pub async fn get_user(&self) -> Result<User, Error> {
        self.local_cache_async({
            struct CurrentUser(Result<User, Error>);

            async move {
                CurrentUser(if let Some(user) = self.message.user.as_ref() {
                    Ok(user.clone())
                } else if let Some(user) = self.cache.get_user(&self.message.author) {
                    Ok(user.clone())
                } else if let Ok(user) = self.http.fetch_user(&self.message.author).await {
                    Ok(user)
                } else {
                    Err(Error::InternalError)
                })
            }
        })
        .await
        .0
        .clone()
    }

    pub async fn get_member(&self) -> Result<Member, Error> {
        self.local_cache_async({
            struct CurrentMember(Result<Member, Error>);

            async move {
                CurrentMember(if let Some(member) = self.message.member.as_ref() {
                    Ok(member.clone())
                } else if let Ok(server) = self.get_current_server() {
                    if let Some(member) = self.cache.get_member(&server.id, &self.message.author) {
                        Ok(member.clone())
                    } else if let Ok(member) = self
                        .http
                        .fetch_member(&server.id, &self.message.author)
                        .await
                    {
                        Ok(member)
                    } else {
                        Err(Error::InternalError)
                    }
                } else {
                    Err(Error::InternalError)
                })
            }
        })
        .await
        .0
        .clone()
    }

    pub async fn get_author_channel_permissions(&self) -> PermissionValue {
        self.local_cache_async(async {
            struct ChannelPermissions(PermissionValue);

            let Ok(user) = self.get_user().await else {
                return ChannelPermissions(0u64.into());
            };
            let member = self.get_member().await;
            let Ok(channel) = self.get_current_channel() else {
                return ChannelPermissions(0u64.into());
            };
            let server = self.get_current_server();

            let mut query =
                user_permissions_query(self.cache.clone(), self.http.clone(), Cow::Owned(user))
                    .channel(Cow::Owned(channel));

            if let Ok(server) = server {
                query = query.server(Cow::Owned(server))
            };

            if let Ok(member) = member {
                query = query.member(Cow::Owned(member))
            };

            ChannelPermissions(calculate_channel_permissions(&mut query).await)
        })
        .await
        .0
    }

    pub async fn get_author_server_permissions(&self) -> PermissionValue {
        self.local_cache_async(async {
            struct ServerPermissions(PermissionValue);

            let Ok(user) = self.get_user().await else {
                return ServerPermissions(0u64.into());
            };
            let member = self.get_member().await;
            let server = self.get_current_server();

            let mut query =
                user_permissions_query(self.cache.clone(), self.http.clone(), Cow::Owned(user));

            if let Ok(server) = server {
                query = query.server(Cow::Owned(server))
            };

            if let Ok(member) = member {
                query = query.member(Cow::Owned(member))
            };

            ServerPermissions(calculate_server_permissions(&mut query).await)
        })
        .await
        .0
    }
}

impl<E, S> Deref for Context<E, S> {
    type Target = MessageContext;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E, S> AsRef<GlobalCache> for Context<E, S> {
    fn as_ref(&self) -> &GlobalCache {
        &self.cache
    }
}

impl<E, S> AsRef<HttpClient> for Context<E, S> {
    fn as_ref(&self) -> &HttpClient {
        &self.http
    }
}

impl<E, S> AsRef<Notifiers> for Context<E, S> {
    fn as_ref(&self) -> &Notifiers {
        &self.notifiers
    }
}

impl<E, S> AsRef<Events> for Context<E, S> {
    fn as_ref(&self) -> &Events {
        &self.events
    }
}
