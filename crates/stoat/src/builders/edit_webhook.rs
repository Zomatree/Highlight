use stoat_models::v0::{DataEditWebhook, FieldsWebhook, Webhook};

use crate::{HttpClient, error::Error};

pub struct EditWebhookBuilder {
    http: HttpClient,
    webhook_id: String,
    token: Option<String>,
    data: DataEditWebhook,
}

impl EditWebhookBuilder {
    pub fn new(http: HttpClient, webhook_id: String, token: Option<String>) -> Self {
        Self {
            http,
            webhook_id,
            token,
            data: DataEditWebhook {
                name: None,
                avatar: None,
                permissions: None,
                remove: Vec::new(),
            },
        }
    }

    pub fn avatar(&mut self, avatar: Option<String>) -> &mut Self {
        if avatar.is_some() {
            self.data.avatar = avatar;
        } else {
            self.data.remove.push(FieldsWebhook::Avatar);
        };

        self
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.data.name = Some(name);

        self
    }

    pub fn permissions(&mut self, permissions: u64) -> &mut Self {
        self.data.permissions = Some(permissions);

        self
    }

    pub async fn build(&self) -> Result<Webhook, Error> {
        if let Some(token) = &self.token {
            self.http
                .edit_webhook_token(&self.webhook_id, &self.data, token)
                .await
        } else {
            self.http.edit_webhook(&self.webhook_id, &self.data).await
        }
    }
}
