use std::time::Duration;

use revolt::{async_trait, commands::CommandHandler, types::{Channel, Message, SendableEmbed}, Context, EventHandler};

use crate::{Error, State, commands::CommandEvents};

#[derive(Clone)]
pub struct Events {
    pub commands: CommandHandler<CommandEvents, Error, State>,
    pub state: State
}

#[async_trait]
impl EventHandler<Error> for Events {
    async fn message(&self, ctx: Context, message: Message) -> Result<(), Error> {
        println!("{message:?}");


        tokio::spawn({
            let commands = self.commands.clone();
            let context = ctx.clone();
            let message = message.clone();

            async move {
                commands.process_commands(context, message).await
            }
        });

        let Some(content) = &message.content else { return Ok(()) };

        let server_id = match ctx.cache.read().await.channels.get(&message.channel).unwrap() {
            Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => server.clone(),
            _ => return Ok(())
        };


        let regexes = self.state.get_keywords(server_id.clone()).await?;

        for (user_id, regex) in regexes {
            println!("{regex:?}");

            if &user_id == &message.author {
                continue
            };

            tokio::spawn({
                let server_id = server_id.clone();
                let channel_id = message.channel.clone();
                let message_id = message.id.clone();
                let content = content.clone();

                let cache = ctx.cache.read().await;
                let server_name =  cache.servers.get(&server_id).unwrap().name.clone();
                let channel_name = cache.channels.get(&channel_id).unwrap().name().unwrap().to_string();
                drop(cache);

                let waiters = ctx.waiters.clone();
                let http = ctx.http.clone();

                async move {
                    if let Some(res) = regex.find(&content) {
                        println!("{res:?}");

                        let msg_fut = waiters.wait_for_message(
                            {
                                let channel_id = channel_id.clone();
                                let user_id = user_id.clone();

                                move |msg| &msg.channel == &channel_id && msg.author == user_id
                            },
                            Some(Duration::from_secs(10))
                        );

                        let typing_fut = waiters.wait_for_typing_start(
                            {
                                let channel_id = channel_id.clone();
                                let user_id = user_id.clone();

                                move |(typing_user_id, typing_channel_id)| typing_channel_id == &channel_id && typing_user_id == &user_id
                            },
                            Some(Duration::from_secs(10)));

                        let should_cancel = tokio::select! {
                            msg = msg_fut => { msg.is_ok() },
                            data = typing_fut => { data.is_ok() }
                        };

                        if should_cancel {
                            return
                        };

                        let mut messages = http.fetch_messages(&channel_id)
                            .limit(5)
                            .nearby(message_id.clone())
                            .build_with_users()
                            .await
                            .unwrap();

                        messages.messages.sort_by(|a, b| a.id.cmp(&b.id));

                        let jump_link = format!("https://app.revolt.chat/server/{server_id}/channel/{channel_id}/{message_id}");
                        let keyword = res.as_str();

                        let built_messages = messages.messages
                            .iter()
                            .map(|message| {
                                let (is_main_message, content) = if &message.id == &message_id {
                                    let mut raw_content = message.content.clone().unwrap();

                                    raw_content.insert_str(res.end(), "**");
                                    raw_content.insert_str(res.start(), "**");

                                    (true, raw_content)
                                } else {
                                    (false, message.content.clone().unwrap_or_default())
                                };

                                let created_at = ulid::Ulid::from_string(&message.id).unwrap().timestamp_ms() / 1000;
                                let timestamp = format!("<t:{created_at}:T>");

                                let user = messages.users.iter().find(|user| &user.id == &message.author).unwrap();

                                let username = if is_main_message {
                                    format!("**{}#{}**", user.username, user.discriminator)
                                } else {
                                    format!("{}#{}", user.username, user.discriminator)
                                };

                                format!("{timestamp} {username}: {content}")
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        let dm_channel = http.open_dm(&user_id)
                            .await
                            .unwrap();

                        http.send_message(dm_channel.id())
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

    async fn ready(&self, ctx: Context) -> Result<(), Error> {
        println!("Ready!");

        Ok(())
    }
}
