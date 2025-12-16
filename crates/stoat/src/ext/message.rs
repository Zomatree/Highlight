use std::time::SystemTime;

use crate::{
    HttpClient, Identifiable, Result,
    builders::{edit_message::EditMessageBuilder, send_message::SendMessageBuilder},
    created_at,
};
use async_trait::async_trait;
use stoat_models::v0::{Message, OptionsUnreact};

#[async_trait]
pub trait MessageExt {
    fn reply(&self, http: &HttpClient, mention: bool) -> SendMessageBuilder;
    fn edit(&self, http: &HttpClient) -> EditMessageBuilder;
    async fn delete(&self, http: &HttpClient) -> Result<()>;
    async fn clear_reactions(&mut self, http: &HttpClient) -> Result<()>;
    async fn pin_message(&mut self, http: &HttpClient) -> Result<()>;
    async fn unpin_message(&mut self, http: &HttpClient) -> Result<()>;
    async fn react(&mut self, http: &HttpClient, emoji: &str) -> Result<()>;
    async fn unreact(&mut self, http: &HttpClient, emoji: &str) -> Result<()>;
    async fn remove_reaction(
        &mut self,
        http: &HttpClient,
        emoji: &str,
        options: &OptionsUnreact,
    ) -> Result<()>;
}

#[async_trait]
impl MessageExt for Message {
    fn reply(&self, http: &HttpClient, mention: bool) -> SendMessageBuilder {
        SendMessageBuilder::new(http.clone(), self.channel.clone()).reply(self.id.clone(), mention)
    }

    fn edit(&self, http: &HttpClient) -> EditMessageBuilder {
        EditMessageBuilder::new(http.clone(), self.channel.clone(), self.id.clone())
    }

    async fn delete(&self, http: &HttpClient) -> Result<()> {
        http.delete_message(&self.channel, &self.id).await
    }

    async fn clear_reactions(&mut self, http: &HttpClient) -> Result<()> {
        http.clear_reactions(&self.channel, &self.id).await?;

        self.reactions.clear();

        Ok(())
    }

    async fn pin_message(&mut self, http: &HttpClient) -> Result<()> {
        http.pin_message(&self.channel, &self.id).await?;

        self.pinned = Some(true);

        Ok(())
    }

    async fn unpin_message(&mut self, http: &HttpClient) -> Result<()> {
        http.unpin_message(&self.channel, &self.id).await?;

        self.pinned = None;

        Ok(())
    }

    async fn react(&mut self, http: &HttpClient, emoji: &str) -> Result<()> {
        http.react_message(&self.channel, &self.id, emoji).await?;

        if let Some(user_id) = http.user_id.clone() {
            self.reactions
                .entry(emoji.to_string())
                .or_default()
                .insert(user_id);
        };

        Ok(())
    }

    async fn unreact(&mut self, http: &HttpClient, emoji: &str) -> Result<()> {
        http.unreact_message(
            &self.channel,
            &self.id,
            emoji,
            &OptionsUnreact {
                user_id: None,
                remove_all: None,
            },
        )
        .await?;

        if let Some(user_id) = &http.user_id {
            if let Some(users) = self.reactions.get_mut(emoji) {
                users.remove(user_id);
            };
        };

        Ok(())
    }

    async fn remove_reaction(
        &mut self,
        http: &HttpClient,
        emoji: &str,
        options: &OptionsUnreact,
    ) -> Result<()> {
        http.unreact_message(&self.channel, &self.id, emoji, options)
            .await?;

        if options.remove_all == Some(true) {
            self.reactions.remove(emoji);
        } else if let Some(user_id) = &options.user_id {
            if let Some(users) = self.reactions.get_mut(emoji) {
                users.remove(user_id);
            };
        } else if let Some(user_id) = &http.user_id {
            if let Some(users) = self.reactions.get_mut(emoji) {
                users.remove(user_id);
            };
        };

        Ok(())
    }
}

impl Identifiable for Message {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id)
    }
}
