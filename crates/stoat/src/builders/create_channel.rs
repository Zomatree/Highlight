use stoat_models::v0::{Channel, DataCreateServerChannel, VoiceInformation};

use crate::{HttpClient, error::Error};

pub struct CreateChannelBuilder {
    http: HttpClient,
    server_id: String,
    data: DataCreateServerChannel,
}

impl CreateChannelBuilder {
    pub fn new(http: HttpClient, server_id: String, name: String) -> Self {
        Self {
            http,
            server_id,
            data: DataCreateServerChannel {
                name,
                ..Default::default()
            },
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.data.description = Some(description);

        self
    }

    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.data.nsfw = Some(nsfw);

        self
    }

    pub fn voice(mut self, voice: VoiceInformation) -> Self {
        self.data.voice = Some(voice);

        self
    }

    pub async fn build(&self) -> Result<Channel, Error> {
        self.http.create_channel(&self.server_id, &self.data).await
    }
}
