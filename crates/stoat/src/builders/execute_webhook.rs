use stoat_models::v0::{
    DataMessageSend, Interactions, Masquerade, Message, ReplyIntent, SendableEmbed,
};

use crate::{HttpClient, error::Error};

pub struct ExecuteWebhookBuilder {
    http: HttpClient,
    webhook_id: String,
    token: String,
    data: DataMessageSend,
}

impl ExecuteWebhookBuilder {
    pub fn new(http: HttpClient, webhook_id: String, token: String) -> Self {
        Self {
            http,
            webhook_id,
            token,
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

    pub fn content(&mut self, content: String) -> &mut Self {
        self.data.content = Some(content);

        self
    }

    pub fn nonce(&mut self, nonce: String) -> &mut Self {
        self.data.nonce = Some(nonce);

        self
    }

    pub fn attachment(&mut self, file_id: String) -> &mut Self {
        self.data.attachments.get_or_insert_default().push(file_id);

        self
    }

    pub fn reply(&mut self, message_id: String, mention: bool) -> &mut Self {
        self.data.replies.get_or_insert_default().push(ReplyIntent {
            id: message_id,
            mention,
            fail_if_not_exists: None,
        });

        self
    }

    pub fn embed(&mut self, embed: SendableEmbed) -> &mut Self {
        self.data.embeds.get_or_insert_default().push(embed);

        self
    }

    pub fn masquerade(&mut self, masquerade: Masquerade) -> &mut Self {
        self.data.masquerade = Some(masquerade);

        self
    }

    pub fn interactions(&mut self, interactions: Interactions) -> &mut Self {
        self.data.interactions = Some(interactions);

        self
    }

    pub fn flags(&mut self, flags: u32) -> &mut Self {
        self.data.flags = Some(flags);

        self
    }

    pub async fn build(&self) -> Result<Message, Error> {
        self.http
            .execute_webhook_token(&self.webhook_id, &self.token, &self.data)
            .await
    }
}
