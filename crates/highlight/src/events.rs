use std::{borrow::Cow, time::Duration};

use stoat::{
    Context, EventHandler, async_trait,
    builders::{fetch_messages::FetchMessagesBuilder, send_message::SendMessageBuilder},
    commands::CommandHandler,
    permissions::{ChannelPermission, calculate_channel_permissions, user_permissions_query},
    types::{Channel, Member, Message, RemovalIntention, SendableEmbed},
};

use crate::{Error, State, commands::CommandEvents};

#[derive(Clone)]
pub struct Events {
    pub commands: CommandHandler<CommandEvents>,
    pub state: State,
}

#[async_trait]
impl EventHandler for Events {
    type Error = Error;

    async fn message(&self, ctx: Context, message: Message) -> Result<(), Error> {
        if message.user.as_ref().is_none_or(|user| user.bot.is_some()) {
            return Ok(());
        };

        tokio::spawn({
            let commands = self.commands.clone();
            let context = ctx.clone();
            let message = message.clone();

            async move { commands.process_commands(context, message).await }
        });

        let Some(content) = &message.content else {
            return Ok(());
        };

        let channel = ctx.cache.get_channel(&message.channel).unwrap();

        let server_id = match &channel {
            Channel::TextChannel { server, .. } => server.clone(),
            _ => return Ok(()),
        };

        let server = ctx.cache.get_server(&server_id).unwrap();

        let regexes = self.state.get_keywords(server_id.clone()).await?;
        let known_not_in_server = self
            .state
            .known_not_in_server
            .read()
            .await
            .get(&server_id)
            .cloned()
            .unwrap_or_default();

        for (user_id, (_, regex)) in regexes {
            if known_not_in_server.contains(&user_id) {
                continue;
            };

            let permissions = {
                let user = if let Some(user) = ctx.cache.get_user(&user_id) {
                    user
                } else if let Ok(user) = ctx.http.fetch_user(&user_id).await {
                    ctx.cache.insert_user(user.clone());

                    user
                } else {
                    self.state
                        .known_not_in_server
                        .write()
                        .await
                        .entry(server_id.clone())
                        .or_default()
                        .insert(user_id.clone());

                    continue;
                };

                let member = if let Some(member) = ctx.cache.get_member(&server_id, &user_id) {
                    member.clone()
                } else if let Ok(member) = ctx.http.fetch_member(&server_id, &user_id).await {
                    ctx.cache.insert_member(member.clone());

                    member
                } else {
                    self.state
                        .known_not_in_server
                        .write()
                        .await
                        .entry(server_id.clone())
                        .or_default()
                        .insert(user_id.clone());

                    continue;
                };

                let mut query =
                    user_permissions_query(ctx.cache.clone(), ctx.http.clone(), Cow::Owned(user))
                        .channel(Cow::Borrowed(&channel))
                        .server(Cow::Borrowed(&server))
                        .member(Cow::Borrowed(&member));

                calculate_channel_permissions(&mut query).await
            };

            if &user_id == &message.author
                || !permissions.has(ChannelPermission::ViewChannel as u64)
            {
                continue;
            };

            tokio::spawn({
                let server_id = server_id.clone();
                let channel_id = message.channel.clone();
                let message_id = message.id.clone();
                let content = content.clone();
                let message_author = message.author.clone();

                let server_name = ctx.cache.get_server(&server_id).unwrap().name;
                let channel_name = ctx
                    .cache
                    .get_channel(&channel_id)
                    .unwrap()
                    .name()
                    .unwrap()
                    .to_string();

                let waiters = ctx.notifiers.clone();
                let http = ctx.http.clone();
                let state = self.state.clone();
                let api_config = ctx.cache.api_config.clone();

                async move {
                    if let Some(captures) = regex.captures(&content) {
                        let group = captures.get(1).unwrap();

                        if state
                            .fetch_blocked_users(user_id.clone())
                            .await
                            .unwrap()
                            .contains(&message_author)
                        {
                            return;
                        };

                        let msg_fut = waiters.wait_for_message(
                            {
                                let channel_id = channel_id.clone();
                                let user_id = user_id.clone();

                                move |msg| &msg.channel == &channel_id && msg.author == user_id
                            },
                            Some(Duration::from_secs(10)),
                        );

                        let typing_fut = waiters.wait_for_typing_start(
                            {
                                let channel_id = channel_id.clone();
                                let user_id = user_id.clone();

                                move |(typing_user_id, typing_channel_id)| {
                                    typing_channel_id == &channel_id && typing_user_id == &user_id
                                }
                            },
                            Some(Duration::from_secs(10)),
                        );

                        let should_cancel = tokio::select! {
                            msg = msg_fut => { msg.is_ok() },
                            data = typing_fut => { data.is_ok() }
                        };

                        if should_cancel {
                            return;
                        };

                        let mut messages =
                            FetchMessagesBuilder::new(http.clone(), channel_id.clone())
                                .limit(5)
                                .nearby(message_id.clone())
                                .build_with_users()
                                .await
                                .unwrap();

                        messages.messages.sort_by(|a, b| a.id.cmp(&b.id));

                        let jump_link = format!(
                            "{}/server/{server_id}/channel/{channel_id}/{message_id}",
                            &api_config.app,
                        );
                        let keyword = group.as_str();

                        let built_messages = messages
                            .messages
                            .iter()
                            .map(|message| {
                                let (is_main_message, content) = if &message.id == &message_id {
                                    let mut raw_content = message.content.clone().unwrap();

                                    raw_content.insert_str(group.end(), "**");
                                    raw_content.insert_str(group.start(), "**");

                                    (true, raw_content)
                                } else {
                                    (false, message.content.clone().unwrap_or_default())
                                };

                                let created_at =
                                    ulid::Ulid::from_string(&message.id).unwrap().timestamp_ms()
                                        / 1000;
                                let timestamp = format!("<t:{created_at}:T>");

                                let user = messages
                                    .users
                                    .iter()
                                    .find(|user| &user.id == &message.author)
                                    .unwrap();

                                let username = if is_main_message {
                                    format!("**{}#{}**", user.username, user.discriminator)
                                } else {
                                    format!("{}#{}", user.username, user.discriminator)
                                };

                                format!("{timestamp} {username}: {content}")
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        let dm_channel = http.open_dm(&user_id).await.unwrap();

                        SendMessageBuilder::new(http.clone(), dm_channel.id().to_string())
                            .content(format!("In [{server_name} › {channel_name}]({jump_link}), you where mentioned with **{keyword}**"))
                            .embed(SendableEmbed {
                                title: Some(keyword.to_string()),
                                description: Some(format!("{built_messages}\n\n[Jump to]({jump_link})")),
                                ..Default::default()
                            })
                            .build()
                            .await
                            .unwrap();
                    }
                }
            });
        }

        Ok(())
    }

    async fn server_member_join(&self, _ctx: Context, member: Member) -> Result<(), Error> {
        if let Some(set) = self
            .state
            .known_not_in_server
            .write()
            .await
            .get_mut(&member.id.server)
        {
            set.remove(&member.id.user);
        };

        Ok(())
    }

    async fn server_member_leave(
        &self,
        _ctx: Context,
        member: Member,
        _reason: RemovalIntention,
    ) -> Result<(), Error> {
        if let Some(set) = self
            .state
            .known_not_in_server
            .write()
            .await
            .get_mut(&member.id.server)
        {
            set.insert(member.id.user);
        };

        Ok(())
    }

    async fn ready(&self, _ctx: Context) -> Result<(), Error> {
        log::info!("Ready!");

        Ok(())
    }
}
