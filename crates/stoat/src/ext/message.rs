use crate::{
    HttpClient, Identifiable, Result,
    builders::{EditMessageBuilder, SendMessageBuilder},
    types::StoatConfig,
};
use async_trait::async_trait;
use stoat_models::v0::{Message, OptionsUnreact};

#[async_trait]
pub trait MessageExt {
    fn reply(&self, http: impl AsRef<HttpClient>, mention: bool) -> SendMessageBuilder;
    fn edit(&self, http: impl AsRef<HttpClient>) -> EditMessageBuilder;
    async fn delete(&self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    async fn clear_reactions(&mut self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    async fn pin_message(&mut self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    async fn unpin_message(&mut self, http: impl AsRef<HttpClient> + Send) -> Result<()>;
    async fn react(&mut self, http: impl AsRef<HttpClient> + Send, emoji: &str) -> Result<()>;
    async fn unreact(&mut self, http: impl AsRef<HttpClient> + Send, emoji: &str) -> Result<()>;
    async fn remove_reaction(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        emoji: &str,
        options: &OptionsUnreact,
    ) -> Result<()>;

    fn jump_link(&self, config: impl AsRef<StoatConfig>) -> String;
}

#[async_trait]
impl MessageExt for Message {
    fn reply(&self, http: impl AsRef<HttpClient>, mention: bool) -> SendMessageBuilder {
        let mut builder = SendMessageBuilder::new(http.as_ref().clone(), self.channel.clone());
        builder.reply(self.id.clone(), mention);
        builder
    }

    fn edit(&self, http: impl AsRef<HttpClient>) -> EditMessageBuilder {
        EditMessageBuilder::new(http.as_ref().clone(), self.channel.clone(), self.id.clone())
    }

    async fn delete(&self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        http.as_ref().delete_message(&self.channel, &self.id).await
    }

    async fn clear_reactions(&mut self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        http.as_ref()
            .clear_reactions(&self.channel, &self.id)
            .await?;

        self.reactions.clear();

        Ok(())
    }

    async fn pin_message(&mut self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        http.as_ref().pin_message(&self.channel, &self.id).await?;

        self.pinned = Some(true);

        Ok(())
    }

    async fn unpin_message(&mut self, http: impl AsRef<HttpClient> + Send) -> Result<()> {
        http.as_ref().unpin_message(&self.channel, &self.id).await?;

        self.pinned = None;

        Ok(())
    }

    async fn react(&mut self, http: impl AsRef<HttpClient> + Send, emoji: &str) -> Result<()> {
        http.as_ref()
            .react_message(&self.channel, &self.id, emoji)
            .await?;

        if let Some(user_id) = http.as_ref().user_id.clone() {
            self.reactions
                .entry(emoji.to_string())
                .or_default()
                .insert(user_id);
        };

        Ok(())
    }

    async fn unreact(&mut self, http: impl AsRef<HttpClient> + Send, emoji: &str) -> Result<()> {
        http.as_ref()
            .unreact_message(
                &self.channel,
                &self.id,
                emoji,
                &OptionsUnreact {
                    user_id: None,
                    remove_all: None,
                },
            )
            .await?;

        if let Some(user_id) = &http.as_ref().user_id {
            if let Some(users) = self.reactions.get_mut(emoji) {
                users.remove(user_id);
            };
        };

        Ok(())
    }

    async fn remove_reaction(
        &mut self,
        http: impl AsRef<HttpClient> + Send,
        emoji: &str,
        options: &OptionsUnreact,
    ) -> Result<()> {
        http.as_ref()
            .unreact_message(&self.channel, &self.id, emoji, options)
            .await?;

        if options.remove_all == Some(true) {
            self.reactions.remove(emoji);
        } else if let Some(user_id) = &options.user_id {
            if let Some(users) = self.reactions.get_mut(emoji) {
                users.remove(user_id);
            };
        } else if let Some(user_id) = &http.as_ref().user_id {
            if let Some(users) = self.reactions.get_mut(emoji) {
                users.remove(user_id);
            };
        };

        Ok(())
    }

    fn jump_link(&self, config: impl AsRef<StoatConfig>) -> String {
        format!(
            "{}/channel/{}/{}",
            &config.as_ref().app,
            &self.channel,
            &self.id
        )
    }
}

impl Identifiable for Message {
    fn id(&self) -> &str {
        &self.id
    }
}
