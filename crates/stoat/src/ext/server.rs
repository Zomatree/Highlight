use std::time::SystemTime;

use async_trait::async_trait;
use stoat_models::v0::{
    BanListResult, Channel, DataCreateRole, DataCreateServerChannel, DataEditRoleRanks, Emoji,
    Invite, Member, Role, Server,
};
use stoat_permissions::DataPermissionsValue;

use crate::{HttpClient, Identifiable, Result, builders::EditServerBuilder, created_at};

#[async_trait]
pub trait ServerExt {
    async fn fetch_member(
        &self,
        http: impl AsRef<HttpClient> + Send,
        user_id: &str,
    ) -> Result<Member>;
    async fn fetch_bans(&self, http: impl AsRef<HttpClient> + Send) -> Result<BanListResult>;
    async fn unban_member(&self, http: impl AsRef<HttpClient> + Send, user_id: &str) -> Result<()>;
    async fn create_channel(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        data: &DataCreateServerChannel,
    ) -> Result<Channel>;
    async fn fetch_emojis(&self, http: impl AsRef<HttpClient> + Send) -> Result<Vec<Emoji>>;
    async fn fetch_invites(&self, http: impl AsRef<HttpClient> + Send) -> Result<Vec<Invite>>;
    async fn set_default_permissions(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        permissions: u64,
    ) -> Result<()>;
    async fn create_role(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        name: String,
    ) -> Result<Role>;
    async fn reorder_roles(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        order: Vec<Role>,
    ) -> Result<()>;
    async fn fetch_role(&self, http: impl AsRef<HttpClient> + Send, role_id: &str) -> Result<Role>;
    async fn edit_server(&self, http: impl AsRef<HttpClient> + Send) -> EditServerBuilder;
}

#[async_trait]
impl ServerExt for Server {
    async fn fetch_member(
        &self,
        http: impl AsRef<HttpClient> + Send,
        user_id: &str,
    ) -> Result<Member> {
        http.as_ref().fetch_member(&self.id, user_id).await
    }

    async fn fetch_bans(&self, http: impl AsRef<HttpClient> + Send) -> Result<BanListResult> {
        http.as_ref().fetch_bans(&self.id).await
    }

    async fn unban_member(&self, http: impl AsRef<HttpClient> + Send, user_id: &str) -> Result<()> {
        http.as_ref().unban_member(&self.id, user_id).await
    }

    async fn create_channel(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        data: &DataCreateServerChannel,
    ) -> Result<Channel> {
        let channel = http.as_ref().create_channel(&self.id, data).await?;

        self.channels.push(channel.id().to_string());

        Ok(channel)
    }

    async fn fetch_emojis(&self, http: impl AsRef<HttpClient> + Send) -> Result<Vec<Emoji>> {
        http.as_ref().fetch_emojis(&self.id).await
    }

    async fn fetch_invites(&self, http: impl AsRef<HttpClient> + Send) -> Result<Vec<Invite>> {
        http.as_ref().fetch_invites(&self.id).await
    }

    async fn set_default_permissions(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        permissions: u64,
    ) -> Result<()> {
        let server = http
            .as_ref()
            .set_default_server_permissions(&self.id, &DataPermissionsValue { permissions })
            .await?;

        *self = server;

        Ok(())
    }

    async fn create_role(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        name: String,
    ) -> Result<Role> {
        let role = http
            .as_ref()
            .create_role(&self.id, &DataCreateRole { name, rank: None })
            .await?;

        self.roles.insert(role.id, role.role.clone());

        Ok(role.role)
    }

    async fn reorder_roles(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        order: Vec<Role>,
    ) -> Result<()> {
        let server = http
            .as_ref()
            .edit_role_positions(
                &self.id,
                &DataEditRoleRanks {
                    ranks: order.into_iter().map(|r| r.id).collect(),
                },
            )
            .await?;

        *self = server;

        Ok(())
    }

    async fn fetch_role(&self, http: impl AsRef<HttpClient> + Send, role_id: &str) -> Result<Role> {
        http.as_ref().fetch_role(&self.id, role_id).await
    }

    async fn edit_server(&self, http: impl AsRef<HttpClient> + Send) -> EditServerBuilder {
        EditServerBuilder::new(http.as_ref().clone(), self.id.clone())
    }
}

impl Identifiable for Server {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id)
    }
}
