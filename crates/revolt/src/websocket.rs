use futures::{SinkExt, StreamExt};
use revolt_database::events::client::{EventV1, Ping};
use serde::Serialize;
use std::{sync::{Arc}, time::Duration};
use tokio::{
    sync::{mpsc::UnboundedSender, Mutex, RwLock},
    time::sleep,
};
use tokio_tungstenite::connect_async;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { token: String },
    BeginTyping { channel: String },
    EndTyping { channel: String },
    Subscribe { server_id: String },
    Ping { data: Ping, responded: Option<()> },
}

use crate::cache::GlobalCache;

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
    global_state: Arc<RwLock<GlobalCache>>,
    token: String,
) {
    let ws = {
        let state = global_state.read().await;

        connect_async(&state.api_config.ws).await.unwrap().0
    };

    let (ws_send, mut ws_receive) = ws.split();

    let ws_send = Arc::new(Mutex::new(ws_send));

    send!(ws_send, &ClientMessage::Authenticate { token }).unwrap();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receive.next().await {
            if let Ok(data) = msg.to_text() {
                let event = serde_json::from_str(data).unwrap();

                if let EventV1::Authenticated = &event {
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
                            Ok::<(), tungstenite::Error>(())
                        }
                    });
                };

                events.send(event).unwrap();
            }
        }
    });
}
