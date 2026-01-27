use stoat_database::iso8601_timestamp::Timestamp;
use stoat_models::v0::{DataMemberEdit, FieldsMember, Member};

use crate::{HttpClient, error::Error};

pub struct EditMemberBuilder {
    http: HttpClient,
    server_id: String,
    member_id: String,
    data: DataMemberEdit,
}

impl EditMemberBuilder {
    pub fn new(http: HttpClient, server_id: String, member_id: String) -> Self {
        Self {
            http,
            server_id,
            member_id,
            data: DataMemberEdit {
                nickname: None,
                avatar: None,
                roles: None,
                timeout: None,
                can_publish: None,
                can_receive: None,
                voice_channel: None,
                remove: Vec::new(),
            },
        }
    }

    pub fn nickname(&mut self, nickname: Option<String>) -> &mut Self {
        if nickname.is_some() {
            self.data.nickname = nickname;
        } else {
            self.data.remove.push(FieldsMember::Nickname);
        };

        self
    }

    pub fn avatar(&mut self, avatar: Option<String>) -> &mut Self {
        if avatar.is_some() {
            self.data.avatar = avatar;
        } else {
            self.data.remove.push(FieldsMember::Avatar);
        };

        self
    }

    pub fn roles(&mut self, roles: Vec<String>) -> &mut Self {
        if !roles.is_empty() {
            self.data.roles = Some(roles);
        } else {
            self.data.remove.push(FieldsMember::Roles);
        };

        self
    }

    pub fn timeout<T: Into<Timestamp>>(&mut self, timestamp: Option<T>) -> &mut Self {
        if let Some(timestamp) = timestamp {
            self.data.timeout = Some(timestamp.into());
        } else {
            self.data.remove.push(FieldsMember::Timeout);
        };

        self
    }

    pub fn can_publish(&mut self, can_publish: Option<bool>) -> &mut Self {
        if can_publish.is_some() {
            self.data.can_publish = can_publish;
        } else {
            self.data.remove.push(FieldsMember::CanPublish);
        };

        self
    }

    pub fn can_receive(&mut self, can_receive: Option<bool>) -> &mut Self {
        if can_receive.is_some() {
            self.data.can_receive = can_receive;
        } else {
            self.data.remove.push(FieldsMember::CanReceive);
        };

        self
    }

    pub fn voice_channel(&mut self, voice_channel: String) -> &mut Self {
        self.data.voice_channel = Some(voice_channel);

        self
    }

    pub async fn build(&self) -> Result<Member, Error> {
        self.http
            .edit_member(&self.server_id, &self.member_id, &self.data)
            .await
    }
}
