use stoat_models::v0::{DataEditRole, FieldsRole, Role};

use crate::{HttpClient, error::Error};

pub struct EditRoleBuilder {
    http: HttpClient,
    server_id: String,
    role_id: String,
    data: DataEditRole,
}

impl EditRoleBuilder {
    pub fn new(http: HttpClient, server_id: String, role_id: String) -> Self {
        Self {
            http,
            server_id,
            role_id,
            data: DataEditRole {
                name: None,
                colour: None,
                hoist: None,
                rank: None,
                remove: Vec::new(),
            },
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.data.name = Some(name);

        self
    }

    pub fn colour(mut self, colour: Option<String>) -> Self {
        if colour.is_some() {
            self.data.colour = colour;
        } else {
            self.data.remove.push(FieldsRole::Colour);
        };

        self
    }

    pub fn hoist(mut self, hoist: bool) -> Self {
        self.data.hoist = Some(hoist);

        self
    }

    pub async fn build(&self) -> Result<Role, Error> {
        self.http
            .edit_role(&self.server_id, &self.role_id, &self.data)
            .await
    }
}
