use std::{fmt::Debug, marker::PhantomData, panic::AssertUnwindSafe, sync::Arc};

use async_recursion::async_recursion;
use futures::{FutureExt, future::join};
use revolt_database::events::client::EventV1;
use tokio::sync::{
    RwLock,
    mpsc::{self, UnboundedReceiver},
};

use crate::{
    Error,
    cache::GlobalCache,
    events::{Context, EventHandler, update_state},
    http::HttpClient,
    waiters::Waiters,
    websocket::run,
};

#[derive(Clone)]
pub struct Client<
    H: EventHandler<E> + Clone + Send + Sync + 'static,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
> {
    pub state: Arc<RwLock<GlobalCache>>,
    pub handler: Arc<H>,
    pub http: HttpClient,
    pub waiters: Waiters,
    _e: PhantomData<E>,
}

impl<
    H: EventHandler<E> + Clone + Send + Sync + 'static,
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
> Client<H, E>
{
    pub async fn new(handler: H, base_url: impl Into<String>) -> Self {
        let http = HttpClient::new(base_url.into(), None);

        let api_config = http.get_root().await.unwrap();

        Self {
            state: Arc::new(RwLock::new(GlobalCache::new(api_config))),
            handler: Arc::new(handler),
            http,
            waiters: Waiters::default(),
            _e: PhantomData,
        }
    }

    pub async fn run(mut self, token: impl Into<String>) {
        let token = token.into();

        self.http.token = Some(token.clone());

        let (sender, receiver) = mpsc::unbounded_channel();

        let handle = tokio::spawn(run(sender, self.state.clone(), token));

        join(handle, self.handle_events(receiver)).await.0.unwrap();
    }

    pub async fn handle_events(self, mut receiver: UnboundedReceiver<EventV1>) {
        while let Some(event) = receiver.recv().await {
            let this = self.clone();

            tokio::spawn(async move {
                this.handle_event(event).await;
            });
        }
    }

    pub async fn handle_event(&self, event: EventV1) {
        {
            let mut state = self.state.write().await;
            update_state(event.clone(), &mut state);
        }

        let context = Context {
            cache: self.state.clone(),
            http: self.http.clone(),
            waiters: self.waiters.clone(),
        };

        let wrapper = AssertUnwindSafe(async {
            if let Err(e) = self.call_event(context.clone(), event).await {
                self.handler.error(context, e).await;
            }
        });

        if let Err(e) = wrapper.catch_unwind().await {
            println!("{e:?}");
        }
    }

    #[async_recursion]
    pub async fn call_event(&self, context: Context, event: EventV1) -> Result<(), E> {
        match event {
            EventV1::Bulk { v } => {
                for e in v {
                    self.call_event(context.clone(), e).await?;
                }

                Ok(())
            }
            EventV1::Authenticated => self.handler.authenticated(context).await,
            EventV1::Ready { .. } => self.handler.ready(context).await,
            EventV1::Message(message) => {
                self.waiters.invoke_message_waiters(&message).await?;

                self.handler.message(context, message).await
            }
            EventV1::ChannelStartTyping { id, user } => {
                self.waiters
                    .invoke_typing_start_waiters(&(id.clone(), user.clone()))
                    .await?;

                self.handler.start_typing(context, id, user).await
            }
            EventV1::ChannelStopTyping { id, user } => {
                self.handler.stop_typing(context, id, user).await
            }
            EventV1::ServerMemberJoin { id, user } => {
                self.handler.server_member_join(context, id, user).await
            }
            EventV1::ServerMemberLeave { id, user, reason } => {
                self.handler
                    .server_member_leave(context, id, user, reason)
                    .await
            }
            _ => Ok(()),
        }
    }
}
