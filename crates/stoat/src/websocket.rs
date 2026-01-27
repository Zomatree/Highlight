use futures::{FutureExt, SinkExt, StreamExt, future::select};
use std::{sync::Arc, time::Duration};
use stoat_database::events::{
    client::{EventV1, Ping},
    server::ClientMessage,
};
use tokio::{
    sync::{
        Mutex,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
    task::AbortHandle,
    time::sleep,
};
use tokio_tungstenite::connect_async_with_config;
use tungstenite::{Message, protocol::WebSocketConfig};

use crate::{Error, cache::GlobalCache};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProgramMessage {
    Close,
}

#[derive(Debug)]
pub(crate) enum EventMessage {
    Client(ClientMessage),
    Program(ProgramMessage),
}

async fn send(
    ws: &Arc<Mutex<impl SinkExt<Message, Error = tungstenite::Error> + Unpin>>,
    event: &ClientMessage,
) -> Result<(), tungstenite::Error> {
    let mut lock = ws.lock().await;

    #[cfg(not(feature = "msgpack"))]
    let message = Message::text(serde_json::to_string(event).unwrap());

    #[cfg(feature = "msgpack")]
    let message = Message::binary(rmp_serde::to_vec_named(event).unwrap());

    lock.send(message).await
}

pub(crate) async fn run(
    events: UnboundedSender<EventV1>,
    client_events: Arc<Mutex<UnboundedReceiver<EventMessage>>>,
    global_state: GlobalCache,
    token: String,
) -> Result<(), Error> {
    let message_format = if cfg!(feature = "msgpack") {
        "msgpack"
    } else {
        "json"
    };

    let uri = format!(
        "{}/?token={token}&format={message_format}",
        &global_state.api_config.ws
    );

    log::debug!("Connecting to websocket with {uri}");

    let mut ws_config = WebSocketConfig::default();
    ws_config.max_frame_size = Some(usize::MAX);
    ws_config.max_message_size = Some(usize::MAX);

    let (ws, _) = connect_async_with_config(uri, Some(ws_config), false)
        .await
        .inspect_err(|e| {
            if let tungstenite::Error::Http(resp) = e
                && let Some(body) = resp.body()
                && let Ok(body) = std::str::from_utf8(body)
            {
                log::error!("Error when attempting to establish websocket connection:\n{body}");
            };
        })?;

    let (ws_send, mut ws_receive) = ws.split();

    let ws_send = Arc::new(Mutex::new(ws_send));

    let server_client = {
        let ws_send = ws_send.clone();

        async move {
            let mut heartbeat_handle: Option<AbortHandle> = None;

            while let Some(msg) = ws_receive.next().await {
                let msg = msg?;

                let event = match msg {
                    Message::Text(data) => {
                        serde_json::from_str(data.as_str()).map_err(|e| e.to_string())
                    }
                    #[cfg(feature = "msgpack")]
                    Message::Binary(data) => {
                        rmp_serde::from_slice(&data).map_err(|e| e.to_string())
                    }
                    msg => {
                        if let Ok(text) = msg.to_text() {
                            log::error!("Unexpected WS message: {text:?}");
                        } else {
                            log::error!("Unexpected WS message: {:?}", msg.into_data());
                        }
                        continue;
                    }
                };

                match event {
                    Ok(event) => {
                        log::debug!("Received event {event:?}");

                        if let EventV1::Authenticated = &event {
                            heartbeat_handle = Some(
                                tokio::spawn({
                                    let ws = ws_send.clone();
                                    let mut i = 0;

                                    async move {
                                        loop {
                                            send(
                                                &ws,
                                                &ClientMessage::Ping {
                                                    data: Ping::Number(i),
                                                    responded: None,
                                                },
                                            )
                                            .await?;
                                            i = i.wrapping_add(1);

                                            sleep(Duration::from_secs(30)).await;
                                        }

                                        #[allow(unreachable_code)]
                                        Ok::<(), Error>(())
                                    }
                                })
                                .abort_handle(),
                            );
                        };

                        events.send(event).map_err(|_| Error::InternalError)?;
                    }
                    Err(e) => {
                        log::error!("Failed to deserialise event: {e:?}");
                    }
                }
            }

            if let Some(handle) = heartbeat_handle {
                handle.abort();
            };

            Ok::<_, Error>(())
        }
    }
    .boxed();

    let client_server = {
        let ws_send = ws_send.clone();

        async move {
            while let Some(message) = client_events.lock().await.recv().await {
                match message {
                    EventMessage::Client(message) => send(&ws_send, &message).await?,
                    EventMessage::Program(ProgramMessage::Close) => return Err(Error::Close),
                }
            }

            Ok::<_, Error>(())
        }
    }
    .boxed();

    select(server_client, client_server).await.into_inner().0
}
