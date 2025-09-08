use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use async_recursion::async_recursion;
use futures::{FutureExt, future::join};
use revolt_database::events::{client::EventV1, server::ClientMessage};
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use crate::{
    Context, Error,
    cache::GlobalCache,
    events::{EventHandler, update_state},
    http::HttpClient,
    notifiers::Notifiers,
    websocket::run,
};

#[derive(Clone)]
pub struct Client<H> {
    pub state: GlobalCache,
    pub handler: Arc<H>,
    pub http: HttpClient,
    pub waiters: Notifiers,
    pub events: Option<Arc<UnboundedSender<ClientMessage>>>,
}

impl<H: EventHandler + Clone + Send + Sync + 'static> Client<H> {
    pub async fn new(handler: H, base_url: impl Into<String>) -> Self {
        let http = HttpClient::new(base_url.into(), None);

        let api_config = http.get_root().await.unwrap();

        Self {
            state: GlobalCache::new(api_config),
            handler: Arc::new(handler),
            http,
            waiters: Notifiers::default(),
            events: None,
        }
    }

    pub async fn run(mut self, token: impl Into<String>) -> Result<(), Error> {
        let token = token.into();

        self.http.token = Some(token.clone());

        let (client_sender, client_receiver) = mpsc::unbounded_channel();
        self.events = Some(Arc::new(client_sender));

        let (sender, receiver) = mpsc::unbounded_channel();

        let handle = tokio::spawn({
            let sender = sender.clone();
            let state = self.state.clone();
            let token = token.clone();
            let client_receiver = Arc::new(Mutex::new(client_receiver));

            async move {
                loop {
                    if let Err(e) = run(
                        sender.clone(),
                        client_receiver.clone(),
                        state.clone(),
                        token.clone(),
                    )
                    .await
                    {
                        println!("{e:?}");
                    }

                    println!("Disconnected! Reconnecting in 10 seconds.");

                    tokio::time::sleep(Duration::from_secs(10)).await;
                }

                #[allow(unreachable_code)]
                Ok(())
            }
        });

        join(handle, self.handle_events(receiver)).await.0.unwrap()
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
        let wrapper = AssertUnwindSafe(async {
            update_state(event.clone(), self.state.clone()).await;

            let context = Context {
                cache: self.state.clone(),
                http: self.http.clone(),
                notifiers: self.waiters.clone(),
                events: self.events.clone().unwrap(),
            };

            if let Err(e) = self.call_event(context.clone(), event).await {
                self.handler.error(context, e).await;
            }
        });

        if let Err(e) = wrapper.catch_unwind().await {
            println!("{e:?}");
        }
    }

    #[async_recursion]
    pub async fn call_event(&self, context: Context, event: EventV1) -> Result<(), H::Error> {
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
            EventV1::ServerMemberJoin { id, member, .. } => {
                self.handler.server_member_join(context, id, member).await
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
