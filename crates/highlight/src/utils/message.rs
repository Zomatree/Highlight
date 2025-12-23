use std::time::Duration;

use stoat::{HttpClient, types::Message};
use tokio::time::sleep;

pub trait MessageExt {
    fn delete_after(self, http: impl AsRef<HttpClient> + Send, duration: Duration) -> Self;
}

impl MessageExt for &Message {
    fn delete_after(self, http: impl AsRef<HttpClient> + Send, duration: Duration) -> Self {
        tokio::spawn({
            let http = http.as_ref().clone();
            let id = self.id.clone();
            let channel = self.channel.clone();

            async move {
                sleep(duration).await;
                let _ = http.delete_message(&channel, &id).await;
            }
        });

        self
    }
}
