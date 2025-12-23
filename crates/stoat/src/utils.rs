use std::time::{Duration, SystemTime};

use stoat_database::events::server::ClientMessage;
use tokio::{select, time::sleep};
use ulid::Ulid;

use crate::context::Events;

pub fn created_at(id: &str) -> SystemTime {
    Ulid::from_string(id).expect("Malformed ID").datetime()
}

pub async fn with_typing<Fut: Future<Output = R>, R>(
    events: &Events,
    channel_id: String,
    fut: Fut,
) -> R {
    let bg = {
        let events = events.clone();
        let channel_id = channel_id.clone();

        async move {
            loop {
                if let Err(e) = events.send_event(ClientMessage::BeginTyping {
                    channel: channel_id.clone(),
                }) {
                    log::error!("Error occurred in with_typing: {e:?}");
                };

                sleep(Duration::from_secs(10)).await;
            }
        }
    };

    select! {
        _ = bg => {
            unreachable!()
        },
        r = fut => {
            if let Err(e) = events.send_event(ClientMessage::EndTyping { channel: channel_id }) {
                log::error!("Error occurred in with_typing: {e:?}");
            };

            r
        }
    }
}
