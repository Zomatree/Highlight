use stoat_models::v0::{DataEditUser, DataUserProfile, FieldsUser, Presence, User};

use crate::{HttpClient, error::Error};

pub struct EditUserBuilder {
    http: HttpClient,
    user_id: String,
    data: DataEditUser,
}

impl EditUserBuilder {
    pub fn new(http: HttpClient, user_id: String) -> Self {
        Self {
            http,
            user_id,
            data: DataEditUser {
                display_name: None,
                avatar: None,
                status: None,
                profile: None,
                badges: None,
                flags: None,
                remove: Vec::new(),
            },
        }
    }

    pub fn display_name(&mut self, display_name: Option<String>) -> &mut Self {
        if display_name.is_some() {
            self.data.display_name = display_name;
        } else {
            self.data.remove.push(FieldsUser::DisplayName);
        };

        self
    }

    pub fn avatar(&mut self, avatar: Option<String>) -> &mut Self {
        if avatar.is_some() {
            self.data.avatar = avatar;
        } else {
            self.data.remove.push(FieldsUser::Avatar);
        };

        self
    }

    pub fn status_text(&mut self, text: Option<String>) -> &mut Self {
        if text.is_some() {
            self.data.status.get_or_insert_default().text = text;
        } else {
            self.data.remove.push(FieldsUser::StatusText);
        };

        self
    }

    pub fn status_presence(&mut self, presence: Option<Presence>) -> &mut Self {
        if presence.is_some() {
            self.data.status.get_or_insert_default().presence = presence;
        } else {
            self.data.remove.push(FieldsUser::StatusPresence);
        };

        self
    }

    pub fn profile_content(&mut self, content: Option<String>) -> &mut Self {
        if content.is_some() {
            self.data
                .profile
                .get_or_insert_with(|| DataUserProfile {
                    content: None,
                    background: None,
                })
                .content = content;
        } else {
            self.data.remove.push(FieldsUser::ProfileContent);
        };

        self
    }

    pub fn profile_background(&mut self, background: Option<String>) -> &mut Self {
        if background.is_some() {
            self.data
                .profile
                .get_or_insert_with(|| DataUserProfile {
                    content: None,
                    background: None,
                })
                .background = background;
        } else {
            self.data.remove.push(FieldsUser::ProfileBackground);
        };

        self
    }

    pub async fn build(&self) -> Result<User, Error> {
        self.http.edit_user(&self.user_id, &self.data).await
    }
}
