use stoat_models::v0::{DataEditMessage, Message, SendableEmbed};

use crate::{HttpClient, error::Error};

pub struct EditMessageBuilder<'a> {
    http: &'a HttpClient,
    channel_id: &'a str,
    message_id: &'a str,
    data: DataEditMessage,
}

impl<'a> EditMessageBuilder<'a> {
    pub fn new(http: &'a HttpClient, channel_id: &'a str, message_id: &'a str) -> Self {
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
            .edit_message(self.channel_id, self.message_id, &self.data)
            .await
    }
}
