use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

use futures::lock::Mutex;
use indexmap::IndexSet;
use paste::paste;
use rand::random;
use stoat_database::events::client::{EventV1, Ping};
use stoat_models::v0::{
    Channel, ChannelVoiceState, Embed, Emoji, FieldsChannel, FieldsMember, FieldsMessage,
    FieldsRole, FieldsServer, FieldsUser, Member, Message, PartialChannel, PartialMember,
    PartialMessage, PartialRole, PartialServer, PartialUser, PartialUserVoiceState,
    RemovalIntention, Role, Server, User, UserVoiceState,
};
use tokio::sync::oneshot;

use crate::Error;

#[derive(Clone)]
struct Waiter<Arg> {
    check: Arc<Box<dyn Fn(&Arg) -> bool + Send + Sync + 'static>>,
    oneshot: Arc<Mutex<Option<oneshot::Sender<Arg>>>>,
}

type WaiterMap<M> = Arc<Mutex<HashMap<usize, Waiter<M>>>>;

macro_rules! generate_notifiers {
    ($($event: ident: $event_arg: ty),* $(,)?) => {
        #[derive(Default, Debug, Clone)]
        pub struct Notifiers {
            $($event: WaiterMap<$event_arg>),*
        }

        impl Notifiers {
            paste! {
                $(
                    pub async fn [<wait_for_ $event>]<
                        F: Fn(&$event_arg) -> bool + Send + Sync + 'static
                    >(
                        &self,
                        check: F,
                        timeout: Option<Duration>
                    ) -> Result<$event_arg, Error> {
                        self.inner_wait(&self.$event, check, timeout).await
                    }

                    pub async fn [<invoke_ $event _waiters>](&self, arg: &$event_arg) {
                        self.inner_invoke(&self.$event, arg).await
                    }

                    pub async fn [<clear_ $event _waiters>](&self) {
                        self.$event.lock().await.clear();
                    }
                )*

                pub async fn clear_all_waiters(&self) {
                    $(
                        self.[<clear_ $event _waiters>]().await;
                    )*
                }
            }
        }
    }
}

generate_notifiers! {
    event: EventV1,
    authenticated: (),
    logout: (),
    pong: Ping,
    ready: (),
    message: Message,
    message_update: (Message, Message, PartialMessage, Vec<FieldsMessage>),
    message_delete: Message,
    message_react: (Message, String, String),
    message_unreact: (Message, String, String),
    message_remove_reaction: (Message, String, IndexSet<String>),
    message_append: (Message, Vec<Embed>),
    user_update: (User, User, PartialUser, Vec<FieldsUser>),
    bulk_message_delete: (String, Vec<String>, Vec<Message>),
    channel_create: Channel,
    channel_update: (Channel, Channel, PartialChannel, Vec<FieldsChannel>),
    channel_delete: Channel,
    channel_group_user_join: (Channel, String),
    channel_group_user_leave: (Channel, String),
    server_create: (Server, Vec<Channel>, Vec<Emoji>, Vec<ChannelVoiceState>),
    server_delete: (Server, Vec<Channel>, Vec<Emoji>, Vec<ChannelVoiceState>),
    server_update: (Server, Server, PartialServer, Vec<FieldsServer>),
    typing_start: (String, String),
    typing_stop: (String, String),
    server_member_join: Member,
    server_member_leave: (Member, RemovalIntention),
    server_member_update: (Member, Member, PartialMember, Vec<FieldsMember>),
    server_role_create: (String, Role),
    server_role_update: (String, Role, Role, PartialRole, Vec<FieldsRole>),
    server_role_delete: (String, Role),
    server_role_ranks_update: (String, Vec<Role>, Vec<Role>),
    user_voice_state_update: (UserVoiceState, UserVoiceState, PartialUserVoiceState),
    user_voice_channel_join: (String, UserVoiceState),
    user_voice_channel_move: (String, String, String, UserVoiceState, UserVoiceState),
    user_voice_channel_leave: (String, UserVoiceState),
    emoji_create: Emoji,
    emoji_delete: Emoji,
}

impl Notifiers {
    async fn inner_wait<F: Fn(&M) -> bool + Send + Sync + 'static, M: Clone>(
        &self,
        waiters: &WaiterMap<M>,
        check: F,
        timeout: Option<Duration>,
    ) -> Result<M, Error> {
        let (sender, receiver) = oneshot::channel();

        let random_value = random();

        {
            let mut lock = waiters.lock().await;

            lock.insert(
                random_value,
                Waiter {
                    check: Arc::new(Box::new(check)),
                    oneshot: Arc::new(Mutex::new(Some(sender))),
                },
            );
        }

        let response = if let Some(timeout) = timeout {
            tokio::time::timeout(timeout, receiver)
                .await
                .map(|res| res.map_err(|_| Error::BrokenChannel))
                .map_err(|_| Error::Timeout)
        } else {
            Ok(receiver.await.map_err(|_| Error::BrokenChannel))
        };

        {
            let mut lock = waiters.lock().await;

            lock.remove(&random_value);
        }

        response?
    }

    async fn inner_invoke<M: Clone + Debug>(&self, waiters: &WaiterMap<M>, value: &M) {
        let lock = waiters.lock().await.clone();

        for (id, waiter) in lock {
            if (waiter.check)(value) {
                if let Some(oneshot) = waiter.oneshot.lock().await.take() {
                    if let Err(e) = oneshot.send(value.clone()) {
                        log::error!("Notifier failed with payload {e:?}")
                    }
                }

                waiters.lock().await.remove(&id);
            }
        }
    }
}

impl AsRef<Notifiers> for Notifiers {
    fn as_ref(&self) -> &Notifiers {
        self
    }
}