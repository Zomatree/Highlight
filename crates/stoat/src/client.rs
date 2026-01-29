use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use futures::FutureExt;
use stoat_database::events::client::EventV1;
use tokio::{
    select,
    sync::{Mutex, mpsc},
};

use crate::{
    Context, Error,
    cache::GlobalCache,
    context::Events,
    events::{EventHandler, update_state},
    http::HttpClient,
    notifiers::Notifiers,
    websocket::run,
};

/// # Stoat Client
///
/// Entrypoint for connecting to the Stoat API.
///
/// Create a client via [`Self::new`] or [`Self::new_with_api_url`] if you are connecting to a 3rd party instance.
/// Clients require an implementation of [`EventHandler`] which contains all the event methods.
///
/// ## Example
///
/// While it may seem like setting up a client requires a lot, the boilerplate required here pays off in larger projects ensuring everything works smoothly together.
/// ```
/// #[derive(Debug, Clone)]
/// pub enum Error {
///     StoatError(stoat::Error),
/// }
///
/// impl From<stoat::Error> for Error {
///     fn from(value: stoat::Error) -> Self {
///         Self::StoatError(value)
///     }
/// }
///
/// #[derive(Clone)]
/// struct Events;
///
/// #[async_trait]
/// impl EventHandler for Events {
///     type Error = Error;
///
///     async fn ready(&self, _context: Context) -> Result<(), Self::Error> {
///         println!("Ready!");
///         Ok(())
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     Client::new(Events).await?.run("TOKEN HERE").await
/// }
/// ```
#[derive(Clone)]
pub struct Client<H> {
    pub state: GlobalCache,
    pub handler: Arc<H>,
    pub http: HttpClient,
    pub waiters: Notifiers,
    events: Option<Events>,
}

impl<H: EventHandler + Clone + Send + Sync + 'static> Client<H> {
    /// Constructs a client with the official instance.
    /// Use [`Self::new_with_api_url`] to customise the Stoat instance.
    pub async fn new(handler: H) -> Result<Self, H::Error> {
        Self::new_with_api_url(handler, "https://api.stoat.chat").await
    }

    /// Constructs a client with a custom Stoat instance.
    pub async fn new_with_api_url(
        handler: H,
        base_url: impl Into<String>,
    ) -> Result<Self, H::Error> {
        let http = HttpClient::new(base_url.into(), None, None).await?;

        Ok(Self {
            state: GlobalCache::new((*http.api_config).clone()),
            handler: Arc::new(handler),
            http,
            waiters: Notifiers::default(),
            events: None,
        })
    }

    /// Connects to the api and sets the current user.
    ///
    /// You will usually not need to call this directly as [`Self::run`] handles this.
    pub async fn start(&mut self, token: impl Into<String>) -> Result<(), H::Error> {
        let token = token.into();

        self.http.token = Some(token.clone());
        self.http.user_id = Some(self.http.fetch_self().await?.id);

        Ok(())
    }

    /// Connects and starts the bot, this connects to the websocket to receive events, this is the main entry point for starting the bot.
    ///
    /// Reconnects are handled automatically.
    pub async fn run(&mut self, token: impl Into<String>) -> Result<(), H::Error> {
        let token = token.into();

        self.start(token.clone()).await?;

        let (client_sender, client_receiver) = mpsc::unbounded_channel();
        self.events = Some(Events(Arc::new(client_sender)));

        let (sender, receiver) = mpsc::unbounded_channel();

        let handle = {
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

                        if let Error::Close = e {
                            return Ok(());
                        }
                    }

                    log::info!("Disconnected! Reconnecting in 10 seconds.");

                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        };

        let res = select! {
            e = handle => e,
            _ = tokio::signal::ctrl_c() => {
                log::info!("Received ctrl+c. exiting.");
                Ok(())
            }
            _ = self.handle_events(receiver) => {
                Ok(())
            }
        };

        self.cleanup().await;

        res
    }

    /// Clears the internal cache.
    pub async fn cleanup(&mut self) {
        self.state.cleanup().await;
        self.waiters.clear_all_waiters().await;
        self.events = None;
    }

    async fn handle_events(&self, mut receiver: mpsc::UnboundedReceiver<EventV1>) {
        while let Some(event) = receiver.recv().await {
            let this = self.clone();

            tokio::spawn(async move {
                this.handle_event(event).await;
            });
        }
    }

    /// Handles a Stoat event with updating the local state, invoking notifiers and calling the event callbacks.
    ///
    /// You shouldn't need to call this yourself unless your mocking events or debugging.
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
