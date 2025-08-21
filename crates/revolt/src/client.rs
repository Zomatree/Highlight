use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use async_recursion::async_recursion;
use futures::future::join;
use revolt_database::events::client::EventV1;
use tokio::sync::{
    RwLock,
    mpsc::{self, UnboundedReceiver},
};

use crate::{
    events::{Context, EventHandler, update_state},
    http::HttpClient,
    state::GlobalState,
    websocket::run,
};

pub struct Client<H: EventHandler<E>, E: Debug + Send + Sync + 'static> {
    pub state: Arc<RwLock<GlobalState>>,
    pub handler: H,
    pub http: HttpClient,
    _e: PhantomData<E>,
}

impl<H: EventHandler<E> + Send + Sync, E: Debug + Send + Sync + 'static> Client<H, E> {
    pub async fn new(handler: H, base_url: impl Into<String>) -> Self {
        let http = HttpClient::new(base_url.into(), None);

        let api_config = http.get_root().await.unwrap();

        Self {
            state: Arc::new(RwLock::new(GlobalState::new(api_config))),
            handler,
            http,
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

    pub async fn handle_events(mut self, mut receiver: UnboundedReceiver<EventV1>) {
        while let Some(event) = receiver.recv().await {
            self.handle_event(event).await;
        }
    }

    pub async fn handle_event(&mut self, event: EventV1) {
        let mut state = self.state.write().await;
        update_state(event.clone(), &mut state);

        let context = Context {
            state: &mut state,
            http: self.http.clone(),
        };

        if let Err(e) = Self::call_event(&mut self.handler, &context, event).await {
            self.handler.error(&context, e).await;
        }
    }

    #[async_recursion]
    pub async fn call_event(
        handler: &mut H,
        context: &Context<'_>,
        event: EventV1,
    ) -> Result<(), E> {
        match event {
            EventV1::Bulk { v } => {
                for e in v {
                    Self::call_event(handler, context, e).await?;
                }

                Ok(())
            }
            EventV1::Authenticated => handler.authenticated(context).await,
            EventV1::Message(message) => handler.message(context, message).await,
            _ => Ok(()),
        }
    }
}
