use std::time::SystemTime;

use async_trait::async_trait;
use bytes::Bytes;
use stoat_models::v0::{
    DataEditUser, FlagResponse, MutualResponse, User, UserProfile, UserVoiceState,
};

use crate::{
    GlobalCache, HttpClient, Identifiable, Result, builders::send_message::SendMessageBuilder,
    created_at,
};

#[async_trait]
pub trait UserExt {
    fn mention(&self) -> String;
    fn name(&self) -> &str;

    fn voice(&self, cache: &GlobalCache) -> Vec<(String, UserVoiceState)>;

    async fn send(&self, http: &HttpClient) -> Result<SendMessageBuilder>;
    async fn edit(&mut self, http: &HttpClient, data: &DataEditUser) -> Result<()>;
    async fn fetch_profile(&self, http: &HttpClient) -> Result<UserProfile>;
    async fn fetch_flags(&self, http: &HttpClient) -> Result<FlagResponse>;
    async fn fetch_mutuals(&self, http: &HttpClient) -> Result<MutualResponse>;
    async fn fetch_default_avatar(&self, http: &HttpClient) -> Result<Bytes>;
}

#[async_trait]
impl UserExt for User {
    fn mention(&self) -> String {
        format!("<@{}>", &self.id)
    }

    fn name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }

    fn voice(&self, cache: &GlobalCache) -> Vec<(String, UserVoiceState)> {
        let mut states = Vec::new();

        for server in cache.servers.iter() {
            for channel in &server.channels {
                if let Some(channel_voice_state) = cache.voice_states.get(channel) {
                    if let Some(user_voice_state) = channel_voice_state
                        .participants
                        .iter()
                        .find(|s| &s.id == &self.id)
                    {
                        states.push((channel.clone(), user_voice_state.clone()));
                    }
                }
            }
        }

        states
    }

    async fn send(&self, http: &HttpClient) -> Result<SendMessageBuilder> {
        let dm_channel = http.open_dm(&self.id).await?;

        Ok(SendMessageBuilder::new(
            http.clone(),
            dm_channel.id().to_string(),
        ))
    }

    async fn edit(&mut self, http: &HttpClient, data: &DataEditUser) -> Result<()> {
        let user = http.edit_user(&self.id, data).await?;

        *self = user;

        Ok(())
    }

    async fn fetch_profile(&self, http: &HttpClient) -> Result<UserProfile> {
        http.fetch_user_profile(&self.id).await
    }

    async fn fetch_flags(&self, http: &HttpClient) -> Result<FlagResponse> {
        http.fetch_user_flags(&self.id).await
    }

    async fn fetch_mutuals(&self, http: &HttpClient) -> Result<MutualResponse> {
        http.fetch_user_mutuals(&self.id).await
    }

    async fn fetch_default_avatar(&self, http: &HttpClient) -> Result<Bytes> {
        http.fetch_default_avatar(&self.id).await
    }
}

impl Identifiable for User {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id)
    }
}
