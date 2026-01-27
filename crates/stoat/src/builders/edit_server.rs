use stoat_models::v0::{Category, DataEditServer, FieldsServer, Server, SystemMessageChannels};

use crate::{HttpClient, error::Error};

pub struct EditServerBuilder {
    http: HttpClient,
    server_id: String,
    data: DataEditServer,
}

impl EditServerBuilder {
    pub fn new(http: HttpClient, server_id: String) -> Self {
        Self {
            http,
            server_id,
            data: DataEditServer {
                name: None,
                description: None,
                icon: None,
                banner: None,
                categories: None,
                system_messages: None,
                flags: None,
                discoverable: None,
                analytics: None,
                remove: Vec::new(),
            },
        }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.data.name = Some(name);

        self
    }

    pub fn description(&mut self, description: Option<String>) -> &mut Self {
        if description.is_some() {
            self.data.description = description;
        } else {
            self.data.remove.push(FieldsServer::Description);
        };

        self
    }

    pub fn icon(&mut self, icon: Option<String>) -> &mut Self {
        if icon.is_some() {
            self.data.icon = icon;
        } else {
            self.data.remove.push(FieldsServer::Icon);
        };

        self
    }

    pub fn banner(&mut self, banner: Option<String>) -> &mut Self {
        if banner.is_some() {
            self.data.banner = banner;
        } else {
            self.data.remove.push(FieldsServer::Banner);
        };

        self
    }

    pub fn categories(&mut self, categories: Option<Vec<Category>>) -> &mut Self {
        if categories.is_some() {
            self.data.categories = categories;
        } else {
            self.data.remove.push(FieldsServer::Categories);
        };

        self
    }

    pub fn system_messages(&mut self, system_messages: Option<SystemMessageChannels>) -> &mut Self {
        if system_messages.is_some() {
            self.data.system_messages = system_messages;
        } else {
            self.data.remove.push(FieldsServer::SystemMessages);
        };

        self
    }

    pub async fn build(&self) -> Result<Server, Error> {
        self.http.edit_server(&self.server_id, &self.data).await
    }
}
