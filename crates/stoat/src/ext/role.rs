use async_trait::async_trait;
use stoat_models::v0::{DataEditRole, DataSetServerRolePermission, Role};
use stoat_permissions::Override;

use crate::{Error, HttpClient, Result};

#[async_trait]
pub trait RoleExt {
    async fn edit(&mut self, http: impl AsRef<HttpClient> + Send, server_id: &str, data: &DataEditRole) -> Result<()>;
    async fn delete(&self, http: impl AsRef<HttpClient> + Send, server_id: &str) -> Result<()>;
    async fn set_role_permissions(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        server_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<()>;
}

#[async_trait]
impl RoleExt for Role {
    async fn edit(&mut self, http: impl AsRef<HttpClient> + Send, server_id: &str, data: &DataEditRole) -> Result<()> {
        let role = http.as_ref().edit_role(server_id, &self.id, data).await?;

        *self = role;

        Ok(())
    }

    async fn delete(&self, http: impl AsRef<HttpClient> + Send, server_id: &str) -> Result<()> {
        http.as_ref().delete_role(server_id, &self.id).await?;

        Ok(())
    }

    async fn set_role_permissions(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        server_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<()> {
        let mut server = http.as_ref()
            .set_role_server_permissions(
                server_id,
                &self.id,
                &DataSetServerRolePermission {
                    permissions: Override { allow, deny },
                },
            )
            .await?;

        *self = server.roles.remove(&self.id).ok_or(Error::InternalError)?;

        Ok(())
    }
}