use std::time::SystemTime;

use async_trait::async_trait;
use stoat_models::v0::{DataBanCreate, DataMemberEdit, Member, ServerBan, UserVoiceState};

use crate::{GlobalCache, HttpClient, Identifiable, Result, created_at};

#[async_trait]
pub trait MemberExt {
    async fn ban(&self, http: impl AsRef<HttpClient> + Send, reason: Option<String>) -> Result<ServerBan>;
    async fn edit(&mut self, http: impl AsRef<HttpClient> + Send, data: &DataMemberEdit) -> Result<()>;
    async fn kick(&self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Option<(String, UserVoiceState)>;
}

#[async_trait]
impl MemberExt for Member {
    async fn ban(&self, http: impl AsRef<HttpClient> + Send, reason: Option<String>) -> Result<ServerBan> {
        http.as_ref().ban_member(&self.id.server, &self.id.user, &DataBanCreate { reason })
            .await
    }

    async fn edit(&mut self, http: impl AsRef<HttpClient> + Send, data: &DataMemberEdit) -> Result<()> {
        let member = http.as_ref()
            .edit_member(&self.id.server, &self.id.user, data)
            .await?;

        *self = member;

        Ok(())
    }

    async fn kick(&self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        http.as_ref().kick_member(&self.id.server, &self.id.user).await
    }

    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Option<(String, UserVoiceState)> {
        let server_channels = cache
            .as_ref()
            .servers
            .get(&self.id.server)
            .map(|s| s.channels.clone())?;

        for channel in server_channels {
            if let Some(channel_voice_state) = cache.as_ref().voice_states.get(&channel) {
                if let Some(user_voice_state) = channel_voice_state
                    .participants
                    .iter()
                    .find(|s| &s.id == &self.id.user)
                {
                    return Some((channel, user_voice_state.clone()));
                }
            }
        }

        None
    }
}

impl Identifiable for Member {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id.user)
    }
}
