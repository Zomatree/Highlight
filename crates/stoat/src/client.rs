use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use futures::{FutureExt, future::join};
use stoat_database::events::{client::EventV1, server::ClientMessage};
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use crate::{
    Context,
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

    pub async fn run(mut self, token: impl Into<String>) -> Result<(), H::Error> {
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
                        log::error!("{e:?}");
                    }

                    log::info!("Disconnected! Reconnecting in 10 seconds.");

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
            let context = Context {
                cache: self.state.clone(),
                http: self.http.clone(),
                notifiers: self.waiters.clone(),
                events: self.events.clone().unwrap(),
            };

            update_state(event.clone(), context.clone(), self.handler.clone()).await
        });

        if let Err(e) = wrapper.catch_unwind().await {
            log::error!("{e:?}");
        }
    }
}
