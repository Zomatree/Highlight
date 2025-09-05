use std::{fmt::Debug, ops::Deref, sync::Arc};

use revolt_models::v0::{Channel, Member, Message, Server, User};
use state::TypeMap;

use crate::{
    Context as MessageContext, Error,
    commands::{Command, Words, handler::Commands},
};

type SendSyncMap = TypeMap![Send + Sync];

#[derive(Debug, Clone)]
pub struct Context<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> {
    pub(crate) inner: MessageContext,
    pub prefix: String,
    pub command: Option<Command<E, S>>,
    pub message: Message,
    pub state: S,
    pub words: Words,
    pub commands: Commands<E, S>,
    pub(crate) local_state: Arc<SendSyncMap>,
}

impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Context<E, S>
{
    pub async fn get_current_channel(&self) -> &Option<Channel> {
        self.local_cache_async(self.cache.get_channel(&self.message.id))
            .await
    }

    pub async fn get_current_server(&self) -> &Option<Server> {
        self.local_cache_async(async {
            let channel = self.get_current_channel().await;

            if let Some(
                Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. },
            ) = channel
            {
                self.cache.get_server(server).await
            } else {
                None
            }
        })
        .await
    }

    pub async fn get_user(&self) -> &Option<User> {
        self.local_cache_async(async {
            if let Some(user) = self.message.user.as_ref() {
                Some(user.clone())
            } else if let Some(user) = self.cache.get_user(&self.message.author).await {
                Some(user.clone())
            } else if let Ok(user) = self.http.fetch_user(&self.message.author).await {
                Some(user)
            } else {
                None
            }
        })
        .await
    }

    pub async fn get_member(&self) -> &Option<Member> {
        self.local_cache_async(async {
            if let Some(member) = self.message.member.as_ref() {
                Some(member.clone())
            } else {
                let server_id = &self.get_current_server().await.as_ref()?.id;

                if let Some(member) = self.cache.get_member(server_id, &self.message.author).await {
                    Some(member.clone())
                } else if let Ok(member) = self
                    .http
                    .fetch_member(server_id, &self.message.author)
                    .await
                {
                    Some(member)
                } else {
                    None
                }
            }
        })
        .await
    }

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
}

impl<E: From<Error> + Clone + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync> Deref
    for Context<E, S>
{
    type Target = MessageContext;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
