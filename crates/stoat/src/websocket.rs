use futures::{SinkExt, StreamExt, future::select};
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
use tokio_tungstenite::connect_async;

use crate::{Error, cache::GlobalCache};

macro_rules! send {
    ($ws: ident, $event: expr) => {
        $ws.lock()
            .await
            .send(tungstenite::Message::text(
                serde_json::to_string($event).unwrap(),
            ))
            .await
    };
}

pub async fn run(
    events: UnboundedSender<EventV1>,
    client_events: Arc<Mutex<UnboundedReceiver<ClientMessage>>>,
    global_state: GlobalCache,
    token: String,
) -> Result<(), Error> {
    let (ws, _) = connect_async(&global_state.api_config.ws).await?;

    let (ws_send, mut ws_receive) = ws.split();

    let ws_send = Arc::new(Mutex::new(ws_send));

    send!(ws_send, &ClientMessage::Authenticate { token })?;

    let server_client = tokio::spawn({
        let ws_send = ws_send.clone();

        async move {
            let mut heartbeat_handle: Option<AbortHandle> = None;

            while let Some(msg) = ws_receive.next().await {
                let msg = msg?;

                if let Ok(data) = msg.to_text() {
                    match serde_json::from_str(data) {
                        Ok(event) => {
                            if let EventV1::Authenticated = &event {
                                heartbeat_handle = Some(
                                    tokio::spawn({
                                        let ws = ws_send.clone();
                                        let mut i = 0;

                                        async move {
                                            loop {
                                                send!(
                                                    ws,
                                                    &ClientMessage::Ping {
                                                        data: Ping::Number(i),
                                                        responded: None
                                                    }
                                                )?;
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
                    };
                } else {
                    log::error!("Unexpected WS message: {:?}", msg.into_data())
                }
            }

            if let Some(handle) = heartbeat_handle {
                handle.abort();
            };

            Ok::<_, Error>(())
        }
    });

    let client_server = tokio::spawn({
        let ws_send = ws_send.clone();

        async move {
            while let Some(message) = client_events.lock().await.recv().await {
                send!(ws_send, &message)?;
            }

            Ok::<_, Error>(())
        }
    });

    select(server_client, client_server)
        .await
        .into_inner()
        .0
        .map_err(|_| Error::InternalError)
        .and_then(|r| r)
}
