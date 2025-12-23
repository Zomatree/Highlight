use stoat_models::v0::SendableEmbed;

pub trait EmbedExt {
    fn icon_url(self, icon_url: String) -> Self;
    fn url(self, url: String) -> Self;
    fn title(self, title: String) -> Self;
    fn description(self, description: String) -> Self;
    fn media(self, media: String) -> Self;
    fn colour(self, colour: String) -> Self;
}

impl EmbedExt for SendableEmbed {
    fn icon_url(mut self, icon_url: String) -> Self {
        self.icon_url = Some(icon_url);
        self
    }

    fn url(mut self, url: String) -> Self {
        self.icon_url = Some(url);
        self
        }

    fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    fn media(mut self, media: String) -> Self {
        self.media = Some(media);
        self
    }

    fn colour(mut self, colour: String) -> Self {
        self.colour = Some(colour);
        self
    }
}