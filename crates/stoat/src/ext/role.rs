use async_trait::async_trait;
use stoat_models::v0::{DataSetServerRolePermission, Role};
use stoat_permissions::Override;

use crate::{Error, HttpClient, Result, builders::EditRoleBuilder};

#[async_trait]
pub trait RoleExt {
    async fn edit(&self, http: impl AsRef<HttpClient> + Send, server_id: String)
    -> EditRoleBuilder;
    async fn delete(&self, http: impl AsRef<HttpClient> + Send, server_id: &str) -> Result<()>;
    async fn set_permissions(
        &self,
        http: impl AsRef<HttpClient> + Send,
        server_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<Role>;
}

#[async_trait]
impl RoleExt for Role {
    async fn edit(
        &self,
        http: impl AsRef<HttpClient> + Send,
        server_id: String,
    ) -> EditRoleBuilder {
        EditRoleBuilder::new(http.as_ref().clone(), server_id, self.id.clone())
    }

    async fn delete(&self, http: impl AsRef<HttpClient> + Send, server_id: &str) -> Result<()> {
        http.as_ref().delete_role(server_id, &self.id).await?;

        Ok(())
    }

    async fn set_permissions(
        &self,
        http: impl AsRef<HttpClient> + Send,
        server_id: &str,
        allow: u64,
        deny: u64,
    ) -> Result<Role> {
        let mut server = http
            .as_ref()
            .set_role_server_permissions(
                server_id,
                &self.id,
                &DataSetServerRolePermission {
                    permissions: Override { allow, deny },
                },
            )
            .await?;

        server.roles.remove(&self.id).ok_or(Error::InternalError)
    }
}
