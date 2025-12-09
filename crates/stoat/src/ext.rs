use crate::{
    Error, HttpClient, builders::{
        edit_message::EditMessageBuilder, fetch_messages::FetchMessagesBuilder,
        send_message::SendMessageBuilder,
    }, GlobalCache,
};
use async_trait::async_trait;
use stoat_models::v0::{Channel, Message};

#[async_trait]
pub trait ChannelExt {
    fn server(&self) -> Option<&str>;
    fn send<'a>(&'a self, http: &'a HttpClient) -> SendMessageBuilder<'a>;
    async fn fetch_message(&self, http: &HttpClient, message_id: &str) -> Result<Message, Error>;
    fn fetch_messages<'a>(&'a self, http: &'a HttpClient) -> FetchMessagesBuilder<'a>;
    async fn join_call(&self, http: &HttpClient, cache: &GlobalCache, node: Option<String>) -> Result<crate::VoiceConnection, Error>;
}

#[async_trait]
impl ChannelExt for Channel {
    fn server(&self) -> Option<&str> {
        match self {
            Channel::TextChannel { server, .. } => Some(server),
            _ => None,
        }
    }

    fn send<'a>(&'a self, http: &'a HttpClient) -> SendMessageBuilder<'a> {
        SendMessageBuilder::new(http, self.id())
    }

    async fn fetch_message(&self, http: &HttpClient, message_id: &str) -> Result<Message, Error> {
        http.fetch_message(self.id(), message_id).await
    }

    fn fetch_messages<'a>(&'a self, http: &'a HttpClient) -> FetchMessagesBuilder<'a> {
        FetchMessagesBuilder::new(http, self.id())
    }

    #[cfg(feature = "voice")]
    async fn join_call(&self, http: &HttpClient, cache: &GlobalCache, node: Option<String>) -> Result<crate::VoiceConnection, Error> {
        let response = http.join_call(self.id(), &stoat_models::v0::DataJoinCall {
            node,
            force_disconnect: None,
            recipients: None,
        }).await?;

        crate::VoiceConnection::connect(cache, &response.url, &response.token).await
    }
}

#[async_trait]
pub trait MessageExt {
    fn reply<'a>(&'a self, http: &'a HttpClient, mention: bool) -> SendMessageBuilder<'a>;
    fn edit<'a>(&'a self, http: &'a HttpClient) -> EditMessageBuilder<'a>;
    async fn delete(&self, http: &HttpClient) -> Result<(), Error>;
}

#[async_trait]
impl MessageExt for Message {
    fn reply<'a>(&'a self, http: &'a HttpClient, mention: bool) -> SendMessageBuilder<'a> {
        SendMessageBuilder::new(http, &self.channel).reply(self.id.clone(), mention)
    }

    fn edit<'a>(&'a self, http: &'a HttpClient) -> EditMessageBuilder<'a> {
        EditMessageBuilder::new(http, &self.channel, &self.id)
    }

    async fn delete(&self, http: &HttpClient) -> Result<(), Error> {
        http.delete_message(&self.channel, &self.id).await
    }
}
