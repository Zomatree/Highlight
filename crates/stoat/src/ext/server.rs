use std::time::SystemTime;

use async_trait::async_trait;
use stoat_models::v0::{
    BanListResult, Channel, DataCreateRole, DataCreateServerChannel, DataEditRoleRanks,
    DataEditServer, DataSetServerRolePermission, Emoji, Invite, Member, Role, Server,
};
use stoat_permissions::{DataPermissionsValue, Override};

use crate::{HttpClient, Identifiable, Result, created_at};

#[async_trait]
pub trait ServerExt {
    async fn fetch_member(&self, http: impl AsRef<HttpClient> + Send, user_id: &str) -> Result<Member>;
    async fn fetch_bans(&self, http: impl AsRef<HttpClient> + Send) -> Result<BanListResult>;
    async fn unban_member(&self, http: impl AsRef<HttpClient> + Send, user_id: &str) -> Result<()>;
    async fn create_channel(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        data: &DataCreateServerChannel,
    ) -> Result<Channel>;
    async fn fetch_emojis(&self, http: impl AsRef<HttpClient> + Send) -> Result<Vec<Emoji>>;
    async fn fetch_invites(&self, http: impl AsRef<HttpClient> + Send) -> Result<Vec<Invite>>;
    async fn set_default_permissions(&mut self, http: impl AsRef<HttpClient> + Send, permissions: u64) -> Result<()>;
    async fn set_role_permissions(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        role_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<()>;
    async fn create_role(&mut self, http: impl AsRef<HttpClient> + Send, data: &DataCreateRole) -> Result<()>;
    async fn reorder_roles(&mut self, http: impl AsRef<HttpClient> + Send, order: Vec<String>) -> Result<()>;
    async fn fetch_role(&self, http: impl AsRef<HttpClient> + Send, role_id: &str) -> Result<Role>;
    async fn edit_server(&mut self, http: impl AsRef<HttpClient> + Send, data: &DataEditServer) -> Result<()>;
}

#[async_trait]
impl ServerExt for Server {
    async fn fetch_member(&self, http: impl AsRef<HttpClient> + Send, user_id: &str) -> Result<Member> {
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

    async fn set_default_permissions(&mut self, http: impl AsRef<HttpClient> + Send, permissions: u64) -> Result<()> {
        let server = http.as_ref()
            .set_default_server_permissions(&self.id, &DataPermissionsValue { permissions })
            .await?;

        *self = server;

        Ok(())
    }

    async fn set_role_permissions(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        role_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<()> {
        let server = http.as_ref()
            .set_role_server_permissions(
                &self.id,
                role_id,
                &DataSetServerRolePermission {
                    permissions: Override { allow, deny },
                },
            )
            .await?;

        *self = server;

        Ok(())
    }

    async fn create_role(&mut self, http: impl AsRef<HttpClient> + Send, data: &DataCreateRole) -> Result<()> {
        let role = http.as_ref().create_role(&self.id, data).await?;

        self.roles.insert(role.id, role.role);

        Ok(())
    }

    async fn reorder_roles(&mut self, http: impl AsRef<HttpClient> + Send, order: Vec<String>) -> Result<()> {
        let server = http.as_ref()
            .edit_role_positions(&self.id, &DataEditRoleRanks { ranks: order })
            .await?;

        *self = server;

        Ok(())
    }

    async fn fetch_role(&self, http: impl AsRef<HttpClient> + Send, role_id: &str) -> Result<Role> {
        http.as_ref().fetch_role(&self.id, role_id).await
    }

    async fn edit_server(&mut self, http: impl AsRef<HttpClient> + Send, data: &DataEditServer) -> Result<()> {
        let server = http.as_ref().edit_server(&self.id, data).await?;

        *self = server;

        Ok(())
    }
}

impl Identifiable for Server {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id)
    }
}
