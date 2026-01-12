use stoat_models::v0::{Channel, DataEditChannel, FieldsChannel, VoiceInformation};

use crate::{HttpClient, error::Error};

pub struct EditChannelBuilder {
    http: HttpClient,
    channel_id: String,
    data: DataEditChannel,
}

impl EditChannelBuilder {
    pub fn new(http: HttpClient, channel_id: String) -> Self {
        Self {
            http,
            channel_id,
            data: DataEditChannel {
                name: None,
                description: None,
                owner: None,
                icon: None,
                nsfw: None,
                archived: None,
                voice: None,
                remove: Vec::new(),
            },
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.data.name = Some(name);

        self
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        if description.is_some() {
            self.data.description = description
        } else {
            self.data.remove.push(FieldsChannel::Description);
        }

        self
    }

    pub fn owner(mut self, owner: String) -> Self {
        self.data.owner = Some(owner);

        self
    }

    pub fn icon(mut self, icon: Option<String>) -> Self {
        if icon.is_some() {
            self.data.icon = icon
        } else {
            self.data.remove.push(FieldsChannel::Icon);
        }

        self
    }

    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.data.nsfw = Some(nsfw);

        self
    }

    pub fn voice(mut self, voice: Option<VoiceInformation>) -> Self {
        if voice.is_some() {
            self.data.voice = voice
        } else {
            self.data.remove.push(FieldsChannel::Voice);
        }

        self
    }

    pub async fn build(&self) -> Result<Channel, Error> {
        self.http
            .edit_channel(&self.channel_id, &self.data)
            .await
    }
}
