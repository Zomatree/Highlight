use std::time::SystemTime;

use async_trait::async_trait;
use stoat_models::v0::{DataBanCreate, DataMemberEdit, Member, Role, ServerBan, UserVoiceState};

use crate::{
    GlobalCache, HttpClient, Identifiable, Result, builders::EditMemberBuilder, created_at,
};

#[async_trait]
pub trait MemberExt {
    async fn ban(
        &self,
        http: impl AsRef<HttpClient> + Send,
        reason: Option<String>,
    ) -> Result<ServerBan>;
    async fn edit(&self, http: impl AsRef<HttpClient> + Send) -> EditMemberBuilder;
    async fn kick(&self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    async fn add_roles(
        &self,
        http: impl AsRef<HttpClient> + Send,
        roles: Vec<Role>,
    ) -> Result<Member>;
    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Option<(String, UserVoiceState)>;
    fn mention(&self) -> String;
}

#[async_trait]
impl MemberExt for Member {
    async fn ban(
        &self,
        http: impl AsRef<HttpClient> + Send,
        reason: Option<String>,
    ) -> Result<ServerBan> {
        http.as_ref()
            .ban_member(&self.id.server, &self.id.user, &DataBanCreate { reason })
            .await
    }

    async fn edit(&self, http: impl AsRef<HttpClient> + Send) -> EditMemberBuilder {
        EditMemberBuilder::new(
            http.as_ref().clone(),
            self.id.server.clone(),
            self.id.user.clone(),
        )
    }

    async fn kick(&self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        http.as_ref()
            .kick_member(&self.id.server, &self.id.user)
            .await
    }

    async fn add_roles(
        &self,
        http: impl AsRef<HttpClient> + Send,
        roles: Vec<Role>,
    ) -> Result<Member> {
        let mut new_roles = self.roles.clone();
        new_roles.extend(roles.into_iter().map(|r| r.id));

        http.as_ref()
            .edit_member(
                &self.id.server,
                &self.id.user,
                &DataMemberEdit {
                    nickname: None,
                    avatar: None,
                    roles: Some(new_roles),
                    timeout: None,
                    can_publish: None,
                    can_receive: None,
                    voice_channel: None,
                    remove: Vec::new(),
                },
            )
            .await
    }

    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Option<(String, UserVoiceState)> {
        let server_channels = cache
            .as_ref()
            .servers
            .get_sync(&self.id.server)
            .map(|s| s.channels.clone())?;

        for channel in server_channels {
            if let Some(channel_voice_state) = cache.as_ref().voice_states.get_sync(&channel) {
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

    fn mention(&self) -> String {
        format!("<@{}>", &self.id.user)
    }
}

impl Identifiable for Member {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id.user)
    }
}
