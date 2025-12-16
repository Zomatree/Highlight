use stoat_models::v0::{DataEditMessage, Message, SendableEmbed};

use crate::{HttpClient, error::Error};

pub struct EditMessageBuilder {
    http: HttpClient,
    channel_id: String,
    message_id: String,
    data: DataEditMessage,
}

impl EditMessageBuilder {
    pub fn new(http: HttpClient, channel_id: String, message_id: String) -> Self {
        Self {
            http,
            channel_id,
            message_id,
            data: DataEditMessage {
                content: None,
                embeds: None,
            },
        }
    }

    pub fn content(mut self, content: String) -> Self {
        self.data.content = Some(content);

        self
    }

    pub fn embed(mut self, embed: SendableEmbed) -> Self {
        self.data.embeds.get_or_insert_default().push(embed);

        self
    }

    pub async fn build(&self) -> Result<Message, Error> {
        self.http
            .edit_message(&self.channel_id, &self.message_id, &self.data)
            .await
    }
}
