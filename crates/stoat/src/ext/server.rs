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
    async fn fetch_member(&self, http: &HttpClient, user_id: &str) -> Result<Member>;
    async fn fetch_bans(&self, http: &HttpClient) -> Result<BanListResult>;
    async fn unban_member(&self, http: &HttpClient, user_id: &str) -> Result<()>;
    async fn create_channel(
        &mut self,
        http: &HttpClient,
        data: &DataCreateServerChannel,
    ) -> Result<Channel>;
    async fn fetch_emojis(&self, http: &HttpClient) -> Result<Vec<Emoji>>;
    async fn fetch_invites(&self, http: &HttpClient) -> Result<Vec<Invite>>;
    async fn set_default_permissions(&mut self, http: &HttpClient, permissions: u64) -> Result<()>;
    async fn set_role_permissions(
        &mut self,
        http: &HttpClient,
        role_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<()>;
    async fn create_role(&mut self, http: &HttpClient, data: &DataCreateRole) -> Result<()>;
    async fn reorder_roles(&mut self, http: &HttpClient, order: Vec<String>) -> Result<()>;
    async fn fetch_role(&self, http: &HttpClient, role_id: &str) -> Result<Role>;
    async fn edit_server(&mut self, http: &HttpClient, data: &DataEditServer) -> Result<()>;
}

#[async_trait]
impl ServerExt for Server {
    async fn fetch_member(&self, http: &HttpClient, user_id: &str) -> Result<Member> {
        http.fetch_member(&self.id, user_id).await
    }

    async fn fetch_bans(&self, http: &HttpClient) -> Result<BanListResult> {
        http.fetch_bans(&self.id).await
    }

    async fn unban_member(&self, http: &HttpClient, user_id: &str) -> Result<()> {
        http.unban_member(&self.id, user_id).await
    }

    async fn create_channel(
        &mut self,
        http: &HttpClient,
        data: &DataCreateServerChannel,
    ) -> Result<Channel> {
        let channel = http.create_channel(&self.id, data).await?;

        self.channels.push(channel.id().to_string());

        Ok(channel)
    }

    async fn fetch_emojis(&self, http: &HttpClient) -> Result<Vec<Emoji>> {
        http.fetch_emojis(&self.id).await
    }

    async fn fetch_invites(&self, http: &HttpClient) -> Result<Vec<Invite>> {
        http.fetch_invites(&self.id).await
    }

    async fn set_default_permissions(&mut self, http: &HttpClient, permissions: u64) -> Result<()> {
        let server = http
            .set_default_server_permissions(&self.id, &DataPermissionsValue { permissions })
            .await?;

        *self = server;

        Ok(())
    }

    async fn set_role_permissions(
        &mut self,
        http: &HttpClient,
        role_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<()> {
        let server = http
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

    async fn create_role(&mut self, http: &HttpClient, data: &DataCreateRole) -> Result<()> {
        let role = http.create_role(&self.id, data).await?;

        self.roles.insert(role.id, role.role);

        Ok(())
    }

    async fn reorder_roles(&mut self, http: &HttpClient, order: Vec<String>) -> Result<()> {
        let server = http
            .edit_role_positions(&self.id, &DataEditRoleRanks { ranks: order })
            .await?;

        *self = server;

        Ok(())
    }

    async fn fetch_role(&self, http: &HttpClient, role_id: &str) -> Result<Role> {
        http.fetch_role(&self.id, role_id).await
    }

    async fn edit_server(&mut self, http: &HttpClient, data: &DataEditServer) -> Result<()> {
        let server = http.edit_server(&self.id, data).await?;

        *self = server;

        Ok(())
    }
}

impl Identifiable for Server {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id)
    }
}
