use async_trait::async_trait;
use stoat_models::v0::Webhook;

use crate::{
    Error, HttpClient, Result,
    builders::{EditWebhookBuilder, ExecuteWebhookBuilder},
};

#[async_trait]
pub trait WebhookExt: Sized {
    async fn from_token(
        http: impl AsRef<HttpClient> + Send,
        webhook_id: &str,
        token: &str,
    ) -> Result<Self>;
    async fn from_url(http: impl AsRef<HttpClient> + Send, url: &str) -> Result<Self>;
    async fn delete(&self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    fn edit(&self, http: impl AsRef<HttpClient>) -> EditWebhookBuilder;
    fn execute(&self, http: impl AsRef<HttpClient>) -> ExecuteWebhookBuilder;
}

#[async_trait]
impl WebhookExt for Webhook {
    async fn from_token(
        http: impl AsRef<HttpClient> + Send,
        webhook_id: &str,
        token: &str,
    ) -> Result<Self> {
        http.as_ref().fetch_webhook_token(webhook_id, token).await
    }

    async fn from_url(http: impl AsRef<HttpClient> + Send, url: &str) -> Result<Self> {
        let mut components = url.split('/').rev();
        let token = components.next().ok_or(Error::InvalidUrl)?;
        let id = components.next().ok_or(Error::InvalidUrl)?;

        Self::from_token(http, id, token).await
    }

    async fn delete(&self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        if let Some(token) = &self.token {
            http.as_ref().delete_webhook_token(&self.id, token).await
        } else {
            http.as_ref().delete_webhook(&self.id).await
        }
    }

    fn edit(&self, http: impl AsRef<HttpClient>) -> EditWebhookBuilder {
        EditWebhookBuilder::new(http.as_ref().clone(), self.id.clone(), self.token.clone())
    }

    fn execute(&self, http: impl AsRef<HttpClient>) -> ExecuteWebhookBuilder {
        ExecuteWebhookBuilder::new(
            http.as_ref().clone(),
            self.id.clone(),
            self.token.clone().expect("Webhook missing token."),
        )
    }
}
