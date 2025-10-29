use reqwest::Method;
use stoat_models::v0::{
    DataMessageSend, Interactions, Masquerade, Message, ReplyIntent, SendableEmbed,
};

use crate::{HttpClient, error::Error};

pub struct SendMessageBuilder<'a> {
    http: &'a HttpClient,
    channel_id: &'a str,
    data: DataMessageSend,
}

impl<'a> SendMessageBuilder<'a> {
    pub fn new(http: &'a HttpClient, channel_id: &'a str) -> Self {
        Self {
            http,
            channel_id,
            data: DataMessageSend {
                content: None,
                nonce: None,
                attachments: None,
                replies: None,
                embeds: None,
                masquerade: None,
                interactions: None,
                flags: None,
            },
        }
    }

    pub fn content(mut self, content: String) -> Self {
        self.data.content = Some(content);

        self
    }

    pub fn nonce(mut self, nonce: String) -> Self {
        self.data.nonce = Some(nonce);

        self
    }

    pub fn attachment(mut self, file_id: String) -> Self {
        self.data.attachments.get_or_insert_default().push(file_id);

        self
    }

    pub fn reply(mut self, reply: ReplyIntent) -> Self {
        self.data.replies.get_or_insert_default().push(reply);

        self
    }

    pub fn embed(mut self, embed: SendableEmbed) -> Self {
        self.data.embeds.get_or_insert_default().push(embed);

        self
    }

    pub fn masquerade(mut self, masquerade: Masquerade) -> Self {
        self.data.masquerade = Some(masquerade);

        self
    }

    pub fn interactions(mut self, interactions: Interactions) -> Self {
        self.data.interactions = Some(interactions);

        self
    }

    pub fn flags(mut self, flags: u32) -> Self {
        self.data.flags = Some(flags);

        self
    }

    pub async fn build(self) -> Result<Message, Error> {
        self.http
            .request(
                Method::POST,
                format!("/channels/{}/messages", &self.channel_id),
            )
            .body(&self.data)
            .response()
            .await
    }
}
