use std::collections::HashMap;

use crate::{
    HttpClient, Identifiable, Result,
    builders::{EditChannelBuilder, FetchMessagesBuilder, SendMessageBuilder},
    context::Events,
    utils,
};
use async_trait::async_trait;
use stoat_models::v0::{
    Channel, CreateWebhookBody, DataDefaultChannelPermissions, DataSetRolePermissions, File,
    Message, OptionsBulkDelete, VoiceInformation, Webhook,
};
use stoat_permissions::{Override, OverrideField};

#[async_trait]
pub trait ChannelExt: Identifiable {
    fn user(&self) -> Option<&str>;
    fn active(&self) -> Option<bool>;
    fn recipients(&self) -> Option<&Vec<String>>;
    fn last_message_id(&self) -> Option<&str>;
    fn owner(&self) -> Option<&str>;
    fn description(&self) -> Option<&str>;
    fn permissions(&self) -> Option<i64>;
    fn nsfw(&self) -> Option<bool>;
    fn default_permissions(&self) -> Option<&OverrideField>;
    fn role_permissions(&self) -> Option<&HashMap<String, OverrideField>>;
    fn voice(&self) -> Option<&VoiceInformation>;
    fn server(&self) -> Option<&str>;
    fn name(&self) -> Option<&str>;
    fn icon(&self) -> Option<&File>;

    fn supports_voice(&self) -> bool;
    fn mention(&self) -> String;

    async fn with_typing<Fut: Future<Output = R> + Send, R>(
        &self,
        events: impl AsRef<Events> + Send,
        fut: Fut,
    ) -> R;

    fn send(&self, http: impl AsRef<HttpClient>) -> SendMessageBuilder;
    async fn fetch_message(
        &self,
        http: impl AsRef<HttpClient> + Send,
        message_id: &str,
    ) -> Result<Message>;
    fn fetch_messages(&self, http: impl AsRef<HttpClient>) -> FetchMessagesBuilder;
    #[cfg(feature = "voice")]
    async fn join_call(
        &self,
        http: impl AsRef<HttpClient> + Send,
        cache: impl AsRef<crate::GlobalCache> + Send,
        node: Option<String>,
    ) -> Result<crate::VoiceConnection>;
    async fn delete(&self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    async fn edit(&self, http: impl AsRef<HttpClient> + Send) -> EditChannelBuilder;
    async fn delete_messages(
        &self,
        http: impl AsRef<HttpClient> + Send,
        options: &OptionsBulkDelete,
    ) -> Result<()>;
    async fn set_default_permissions(
        &self,
        http: impl AsRef<HttpClient> + Send,
        data: &DataDefaultChannelPermissions,
    ) -> Result<Channel>;
    async fn set_role_permissions(
        &self,
        http: impl AsRef<HttpClient> + Send,
        role_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<Channel>;
    async fn create_webhook(
        &self,
        http: impl AsRef<HttpClient> + Send,
        data: &CreateWebhookBody,
    ) -> Result<Webhook>;
}

#[async_trait]
impl ChannelExt for Channel {
    fn user(&self) -> Option<&str> {
        match self {
            Channel::SavedMessages { user, .. } => Some(user),
            _ => None,
        }
    }

    fn active(&self) -> Option<bool> {
        match self {
            Channel::DirectMessage { active, .. } => Some(*active),
            _ => None,
        }
    }

    fn recipients(&self) -> Option<&Vec<String>> {
        match self {
            Channel::DirectMessage { recipients, .. } | Channel::Group { recipients, .. } => {
                Some(recipients)
            }
            _ => None,
        }
    }

    fn last_message_id(&self) -> Option<&str> {
        match self {
            Channel::DirectMessage {
                last_message_id, ..
            }
            | Channel::TextChannel {
                last_message_id, ..
            }
            | Channel::Group {
                last_message_id, ..
            } => last_message_id.as_deref(),
            _ => None,
        }
    }

    fn owner(&self) -> Option<&str> {
        match self {
            Channel::Group { owner, .. } => Some(owner),
            _ => None,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Channel::TextChannel { description, .. } | Channel::Group { description, .. } => {
                description.as_deref()
            }
            _ => None,
        }
    }

    fn permissions(&self) -> Option<i64> {
        match self {
            Channel::Group { permissions, .. } => *permissions,
            _ => None,
        }
    }

    fn nsfw(&self) -> Option<bool> {
        match self {
            Channel::TextChannel { nsfw, .. } | Channel::Group { nsfw, .. } => Some(*nsfw),
            _ => None,
        }
    }

    fn default_permissions(&self) -> Option<&OverrideField> {
        match self {
            Channel::TextChannel {
                default_permissions,
                ..
            } => default_permissions.as_ref(),
            _ => None,
        }
    }

    fn role_permissions(&self) -> Option<&HashMap<String, OverrideField>> {
        match self {
            Channel::TextChannel {
                role_permissions, ..
            } => Some(role_permissions),
            _ => None,
        }
    }

    fn voice(&self) -> Option<&VoiceInformation> {
        match self {
            Channel::TextChannel { voice, .. } => voice.as_ref(),
            _ => None,
        }
    }

    fn supports_voice(&self) -> bool {
        match self {
            Channel::DirectMessage { .. }
            | Channel::Group { .. }
            | Channel::SavedMessages { .. } => true,
            Channel::TextChannel { voice, .. } => voice.is_some(),
        }
    }

    fn server(&self) -> Option<&str> {
        match self {
            Channel::TextChannel { server, .. } => Some(server),
            _ => None,
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            Channel::SavedMessages { .. } => Some("Saved Messages"),
            Channel::DirectMessage { .. } => None,
            Channel::Group { name, .. } | Channel::TextChannel { name, .. } => Some(name),
        }
    }

    fn icon(&self) -> Option<&File> {
        match self {
            Channel::Group { icon, .. } | Channel::TextChannel { icon, .. } => icon.as_ref(),
            _ => None,
        }
    }

    fn mention(&self) -> String {
        format!("<#{}>", self.id())
    }

    async fn with_typing<Fut: Future<Output = R> + Send, R>(
        &self,
        events: impl AsRef<Events> + Send,
        fut: Fut,
    ) -> R {
        utils::with_typing(events.as_ref(), self.id().to_string(), fut).await
    }

    fn send(&self, http: impl AsRef<HttpClient>) -> SendMessageBuilder {
        SendMessageBuilder::new(http.as_ref().clone(), self.id().to_string())
    }

    async fn fetch_message(
        &self,
        http: impl AsRef<HttpClient> + Send,
        message_id: &str,
    ) -> Result<Message> {
        http.as_ref().fetch_message(self.id(), message_id).await
    }

    fn fetch_messages(&self, http: impl AsRef<HttpClient>) -> FetchMessagesBuilder {
        FetchMessagesBuilder::new(http.as_ref().clone(), self.id().to_string())
    }

    #[cfg(feature = "voice")]
    async fn join_call(
        &self,
        http: impl AsRef<HttpClient> + Send,
        cache: impl AsRef<crate::GlobalCache> + Send,
        node: Option<String>,
    ) -> Result<crate::VoiceConnection> {
        let response = http
            .as_ref()
            .join_call(
                self.id(),
                &stoat_models::v0::DataJoinCall {
                    node,
                    force_disconnect: None,
                    recipients: None,
                },
            )
            .await?;

        crate::VoiceConnection::connect(cache.as_ref(), &response.url, &response.token).await
    }

    async fn delete(&self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        http.as_ref().delete_channel(self.id()).await
    }

    async fn edit(&self, http: impl AsRef<HttpClient> + Send) -> EditChannelBuilder {
        EditChannelBuilder::new(http.as_ref().clone(), self.id().to_string())
    }

    async fn delete_messages(
        &self,
        http: impl AsRef<HttpClient> + Send,
        options: &OptionsBulkDelete,
    ) -> Result<()> {
        http.as_ref().delete_messages(self.id(), options).await
    }

    async fn set_default_permissions(
        &self,
        http: impl AsRef<HttpClient> + Send,
        data: &DataDefaultChannelPermissions,
    ) -> Result<Channel> {
        http.as_ref()
            .set_default_channel_permissions(self.id(), data)
            .await
    }

    async fn set_role_permissions(
        &self,
        http: impl AsRef<HttpClient> + Send,
        role_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<Channel> {
        http.as_ref()
            .set_role_channel_permissions(
                self.id(),
                role_id,
                &DataSetRolePermissions {
                    permissions: Override { allow, deny },
                },
            )
            .await
    }

    async fn create_webhook(
        &self,
        http: impl AsRef<HttpClient> + Send,
        data: &CreateWebhookBody,
    ) -> Result<Webhook> {
        http.as_ref().create_webhook(self.id(), data).await
    }
}

impl Identifiable for Channel {
    fn id(&self) -> &str {
        Channel::id(&self)
    }
}
