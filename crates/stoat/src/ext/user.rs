use async_trait::async_trait;
use bytes::Bytes;
use stoat_models::v0::{
    DataEditUser, FlagResponse, MutualResponse, User, UserProfile, UserVoiceState,
};

use crate::{FileExt, GlobalCache, HttpClient, Identifiable, Result, builders::SendMessageBuilder};

#[async_trait]
pub trait UserExt: Identifiable {
    fn mention(&self) -> String;
    fn name(&self) -> &str;

    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Vec<(String, UserVoiceState)>;

    async fn send(&self, http: impl AsRef<HttpClient> + Send) -> Result<SendMessageBuilder>;
    async fn edit(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        data: &DataEditUser,
    ) -> Result<()>;
    async fn fetch_profile(&self, http: impl AsRef<HttpClient> + Send) -> Result<UserProfile>;
    async fn fetch_flags(&self, http: impl AsRef<HttpClient> + Send) -> Result<FlagResponse>;
    async fn fetch_mutuals(&self, http: impl AsRef<HttpClient> + Send) -> Result<MutualResponse>;
    async fn fetch_default_avatar(&self, http: impl AsRef<HttpClient> + Send) -> Result<Bytes>;

    fn avatar_url(&self, http: impl AsRef<HttpClient> + Send) -> String;
    fn default_avatar_url(&self, http: impl AsRef<HttpClient> + Send) -> String;
}

#[async_trait]
impl UserExt for User {
    fn mention(&self) -> String {
        format!("<@{}>", &self.id)
    }

    fn name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }

    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Vec<(String, UserVoiceState)> {
        let mut states = Vec::new();

        cache.as_ref().servers.iter_sync(|_, server| {
            for channel in &server.channels {
                if let Some(channel_voice_state) = cache.as_ref().voice_states.get_sync(channel) {
                    if let Some(user_voice_state) = channel_voice_state
                        .participants
                        .iter()
                        .find(|s| &s.id == &self.id)
                    {
                        states.push((channel.clone(), user_voice_state.clone()));
                    }
                }
            }

            true
        });

        states
    }

    async fn send(&self, http: impl AsRef<HttpClient> + Send) -> Result<SendMessageBuilder> {
        let dm_channel = http.as_ref().open_dm(&self.id).await?;

        Ok(SendMessageBuilder::new(
            http.as_ref().clone(),
            dm_channel.id().to_string(),
        ))
    }

    async fn edit(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        data: &DataEditUser,
    ) -> Result<()> {
        let user = http.as_ref().edit_user(&self.id, data).await?;

        *self = user;

        Ok(())
    }

    async fn fetch_profile(&self, http: impl AsRef<HttpClient> + Send) -> Result<UserProfile> {
        http.as_ref().fetch_user_profile(&self.id).await
    }

    async fn fetch_flags(&self, http: impl AsRef<HttpClient> + Send) -> Result<FlagResponse> {
        http.as_ref().fetch_user_flags(&self.id).await
    }

    async fn fetch_mutuals(&self, http: impl AsRef<HttpClient> + Send) -> Result<MutualResponse> {
        http.as_ref().fetch_user_mutuals(&self.id).await
    }

    async fn fetch_default_avatar(&self, http: impl AsRef<HttpClient> + Send) -> Result<Bytes> {
        http.as_ref().fetch_default_avatar(&self.id).await
    }

    fn avatar_url(&self, http: impl AsRef<HttpClient> + Send) -> String {
        self.avatar
            .as_ref()
            .map(|file| file.url(http.as_ref(), false))
            .unwrap_or_else(|| self.default_avatar_url(http.as_ref()))
    }

    fn default_avatar_url(&self, http: impl AsRef<HttpClient> + Send) -> String {
        format!("{}/users/{}/default_avatar", &http.as_ref().base, &self.id)
    }
}

impl Identifiable for User {
    fn id(&self) -> &str {
        &self.id
    }
}
