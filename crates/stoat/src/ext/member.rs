use async_trait::async_trait;
use stoat_models::v0::{Channel, DataBanCreate, DataMemberEdit, Member, Role, ServerBan, UserVoiceState};

use crate::{GlobalCache, HttpClient, Identifiable, Result, builders::EditMemberBuilder};

#[async_trait]
pub trait MemberExt {
    /// Bans a member from its server.
    async fn ban(
        &self,
        http: impl AsRef<HttpClient> + Send,
        reason: Option<String>,
    ) -> Result<ServerBan>;

    /// Edits a member.
    fn edit(&self, http: impl AsRef<HttpClient>) -> EditMemberBuilder;

    /// Kicks a member from its server.
    async fn kick(&self, http: impl AsRef<HttpClient> + Send) -> Result<()>;

    /// Bans a member from its server.
    async fn add_roles(
        &self,
        http: impl AsRef<HttpClient> + Send,
        roles: &[Role],
    ) -> Result<Member>;

    /// Gets the current voice channel and user voice state for a member if they are connected to a voice channel in this server
    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Option<(Channel, UserVoiceState)>;

    /// Formats a user mention for the member.
    ///
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

    fn edit(&self, http: impl AsRef<HttpClient>) -> EditMemberBuilder {
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
        roles: &[Role],
    ) -> Result<Member> {
        let mut new_roles = self.roles.clone();
        new_roles.extend(roles.iter().map(|r| r.id.clone()));

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

    fn voice(&self, cache: impl AsRef<GlobalCache>) -> Option<(Channel, UserVoiceState)> {
        let server_channels = cache
            .as_ref()
            .servers
            .get_sync(&self.id.server)
            .map(|s| s.channels.clone())?;

        for channel_id in server_channels {
            if let Some(channel_voice_state) = cache.as_ref().voice_states.get_sync(&channel_id) {
                if let Some(user_voice_state) = channel_voice_state
                    .participants
                    .iter()
                    .find(|s| &s.id == &self.id.user)
                {
                    if let Some(channel) = cache.as_ref().get_channel(&channel_id) {
                        return Some((channel, user_voice_state.clone()));
                    }
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
    fn id(&self) -> &str {
        &self.id.user
    }
}
