use std::{fmt::Debug, panic::AssertUnwindSafe, sync::Arc};

use async_trait::async_trait;
use futures::FutureExt;
use stoat_database::events::client::{EventV1, Ping};
use stoat_models::v0::{
    Channel, ChannelVoiceState, Embed, Emoji, FieldsChannel, FieldsMember, FieldsMessage,
    FieldsRole, FieldsServer, FieldsUser, Member, Message, PartialChannel, PartialMember,
    PartialMessage, PartialRole, PartialServer, PartialUser, PartialUserVoiceState,
    RelationshipStatus, RemovalIntention, Role, Server, User, UserVoiceState,
};

use crate::{Context, Error};

macro_rules! set_enum_varient_values {
    ($enum: ident, $key: ident, $value: expr, ($($varient: path),+)) => {
        match $enum {
            $($varient { $key, .. } )|+ => { *$key = $value },
            _ => {}
        }
    };
}

macro_rules! update_enum_partial {
    ($value: ident, $data: expr, $key: ident, ($($varient: path),+)) => {
        if let Some(new_value) = $data.$key {
            set_enum_varient_values!($value, $key, new_value, ($($varient),+))
        }
    };

    (optional, $value: ident, $data: expr, $key: ident, ($($varient: path),+)) => {
        set_enum_varient_values!($value, $key, $data.$key, ($($varient),+))
    };
}

macro_rules! update_multi_enum_partial {
    ($value: ident, $data: expr, ($( $( $(@$optional:tt)? optional )? ($key: ident, ($($varient: path),+))),+ $(,)?)) => {
        $(update_enum_partial!($( $($optional)? optional,)? $value, $data, $key, ($($varient),+)));+
    };
}

macro_rules! handle_event {
    ($handler: ident, $context: ident, $event: ident, ($($arg: expr),*)) => {
        {
            let wrapper = AssertUnwindSafe(async {
                if let Err(e) = $handler.$event($context.clone(), $($arg),*).await {
                    $handler.error($context, e).await;
                }
            });

            if let Err(e) = wrapper.catch_unwind().await {
                log::error!("{e:?}");
            };
        }
    };
}

pub async fn update_state<H: EventHandler + Clone + Send + Sync + 'static>(
    event: EventV1,
    context: Context,
    handler: Arc<H>,
) {
    match event {
        EventV1::Bulk { v } => {
            for e in v {
                Box::pin(update_state(e, context.clone(), handler.clone())).await;
            }
        }
        EventV1::Authenticated => {
            handle_event!(handler, context, authenticated, ())
        }
        EventV1::Logout => {}
        EventV1::Pong { .. } => {}
        EventV1::Ready {
            users,
            servers,
            channels,
            members,
            emojis: _,
            user_settings: _,
            channel_unreads: _,
            policy_changes: _,
            voice_states,
        } => {
            for user in users.into_iter().flatten() {
                if user.relationship == RelationshipStatus::User {
                    context
                        .cache
                        .current_user_id
                        .write()
                        .await
                        .replace(user.id.clone());
                };

                context.cache.insert_user(user).await;
            }

            for server in servers.into_iter().flatten() {
                context.cache.insert_server(server).await;
            }

            for channel in channels.into_iter().flatten() {
                context.cache.insert_channel(channel).await;
            }

            for member in members.into_iter().flatten() {
                context.cache.insert_member(member).await;
            }

            for voice_state in voice_states.into_iter().flatten() {
                context.cache.insert_voice_state(voice_state).await;
            }

            handle_event!(handler, context, ready, ())
        }
        EventV1::Message(message) => {
            context.cache.insert_message(message.clone()).await;
            context.notifiers.invoke_message_waiters(&message).await;

            handle_event!(handler, context, message, (message))
        }
        EventV1::MessageUpdate {
            id,
            channel: _,
            data,
            clear,
        } => {
            if let Some((before, after)) = context
                .cache
                .update_message_with(&id, |message| {
                    let before = message.clone();

                    message.apply_options(data.clone());

                    for field in &clear {
                        match field {
                            FieldsMessage::Pinned => message.pinned = None,
                        }
                    }

                    (before, message.clone())
                })
                .await
            {
                handle_event!(
                    handler,
                    context,
                    message_update,
                    (before, after, data, clear)
                )
            }
        }
        EventV1::MessageDelete { id, channel: _ } => {
            if let Some(message) = context.cache.remove_message(&id).await {
                handle_event!(handler, context, message_delete, (message))
            }
        }
        EventV1::MessageReact {
            id,
            channel_id: _,
            user_id,
            emoji_id,
        } => {
            context
                .cache
                .update_message_with(&id, |message| {
                    message
                        .reactions
                        .entry(emoji_id)
                        .or_default()
                        .insert(user_id);
                })
                .await;
        }
        EventV1::MessageUnreact {
            id,
            channel_id: _,
            user_id,
            emoji_id,
        } => {
            context
                .cache
                .update_message_with(&id, |message| {
                    if let Some(users) = message.reactions.get_mut(&emoji_id) {
                        users.remove(&user_id);

                        if users.is_empty() {
                            message.reactions.remove(&emoji_id);
                        };
                    }
                })
                .await;
        }
        EventV1::MessageRemoveReaction {
            id,
            channel_id: _,
            emoji_id,
        } => {
            context
                .cache
                .update_message_with(&id, |message| {
                    message.reactions.remove(&emoji_id);
                })
                .await;
        }
        EventV1::UserUpdate {
            id, data, clear, ..
        } => {
            if let Some((before, after)) = context
                .cache
                .update_user_with(&id, |user| {
                    let before = user.clone();

                    user.apply_options(data.clone());

                    for field in &clear {
                        match field {
                            FieldsUser::Avatar => user.avatar = None,
                            FieldsUser::StatusText => {
                                if let Some(status) = user.status.as_mut() {
                                    status.text = None
                                };
                            }
                            FieldsUser::StatusPresence => {
                                if let Some(status) = user.status.as_mut() {
                                    status.presence = None
                                };
                            }
                            FieldsUser::DisplayName => user.display_name = None,
                            _ => {}
                        }
                    }

                    (before, user.clone())
                })
                .await
            {
                handle_event!(handler, context, user_update, (before, after, data, clear))
            }
        }
        EventV1::BulkMessageDelete { channel, ids } => {
            let mut channel_messages = context.cache.messages.write().await;

            let mut i = 0;
            let end = channel_messages.len();

            let mut found_messages = Vec::new();

            while i < channel_messages.len() - end {
                if ids.contains(&channel_messages[i].id) {
                    found_messages.push(channel_messages.remove(i).unwrap());
                } else {
                    i += 1;
                };
            }

            drop(channel_messages);

            handle_event!(
                handler,
                context,
                bulk_message_delete,
                (channel, ids, found_messages)
            )
        }
        EventV1::ChannelAck { .. } => {}
        EventV1::ChannelCreate(channel) => {
            if let Channel::TextChannel { id, server, .. } = &channel {
                context
                    .cache
                    .update_server_with(&server, |server| server.channels.push(id.clone()))
                    .await;
            };

            context.cache.insert_channel(channel.clone()).await;

            handle_event!(handler, context, channel_create, (channel))
        }
        EventV1::ChannelDelete { id } => {
            if let Some(channel) = context.cache.remove_channel(&id).await {
                if let Channel::TextChannel { id, server, .. } = &channel {
                    context
                        .cache
                        .update_server_with(&server, |server| {
                            server.channels.retain(|c_id| c_id != id)
                        })
                        .await;
                };

                handle_event!(handler, context, channel_delete, (channel))
            }
        }
        EventV1::ChannelGroupJoin { id, user } => {
            if let Some(channel) = context
                .cache
                .update_channel_with(&id, |channel| {
                    if let Channel::Group { recipients, .. } = channel {
                        recipients.push(user.clone())
                    };

                    channel.clone()
                })
                .await
            {
                handle_event!(handler, context, channel_group_user_join, (channel, user))
            }
        }
        EventV1::ChannelGroupLeave { id, user } => {
            if let Some(channel) = context
                .cache
                .update_channel_with(&id, |channel| {
                    if let Channel::Group { recipients, .. } = channel {
                        recipients.retain(|u_id| u_id != &user)
                    };

                    channel.clone()
                })
                .await
            {
                handle_event!(handler, context, channel_group_user_leave, (channel, user))
            }
        }
        EventV1::ChannelUpdate { id, data, clear } => {
            if let Some((before, after)) = context
                .cache
                .update_channel_with(&id, |channel| {
                    let before = channel.clone();

                    update_multi_enum_partial!(
                        channel,
                        data.clone(),
                        (
                            (name, (Channel::TextChannel)),
                            (owner, (Channel::Group)),
                            optional(description, (Channel::Group, Channel::TextChannel)),
                            optional(icon, (Channel::Group, Channel::TextChannel)),
                            (nsfw, (Channel::Group, Channel::TextChannel)),
                            (active, (Channel::DirectMessage)),
                            optional(permissions, (Channel::Group)),
                            (role_permissions, (Channel::TextChannel)),
                            optional(default_permissions, (Channel::TextChannel)),
                            optional(
                                last_message_id,
                                (Channel::DirectMessage, Channel::Group, Channel::TextChannel)
                            )
                        )
                    );

                    for field in &clear {
                        match field {
                            FieldsChannel::Description => set_enum_varient_values!(
                                channel,
                                description,
                                None,
                                (Channel::Group, Channel::TextChannel)
                            ),
                            FieldsChannel::Icon => set_enum_varient_values!(
                                channel,
                                icon,
                                None,
                                (Channel::Group, Channel::TextChannel)
                            ),
                            FieldsChannel::DefaultPermissions => set_enum_varient_values!(
                                channel,
                                default_permissions,
                                None,
                                (Channel::TextChannel)
                            ),
                            FieldsChannel::Voice => set_enum_varient_values!(
                                channel,
                                voice,
                                None,
                                (Channel::TextChannel)
                            ),
                        }
                    }

                    (before, channel.clone())
                })
                .await
            {
                handle_event!(
                    handler,
                    context,
                    channel_update,
                    (before, after, data, clear)
                )
            }
        }
        EventV1::MessageAppend {
            id,
            channel: _,
            append,
        } => {
            if let Some(message) = context
                .cache
                .update_message_with(&id, |message| {
                    if let Some(embeds) = append.embeds.clone() {
                        message.embeds.get_or_insert_default().extend(embeds);
                    }

                    message.clone()
                })
                .await
            {
                handle_event!(
                    handler,
                    context,
                    message_append,
                    (message, append.embeds.unwrap_or_default())
                )
            }
        }
        EventV1::ServerCreate {
            id: _,
            server,
            channels,
            emojis,
            voice_states,
        } => {
            context.cache.insert_server(server.clone()).await;

            for channel in channels.clone() {
                context.cache.insert_channel(channel).await
            }

            handle_event!(
                handler,
                context,
                server_create,
                (server, channels, emojis, voice_states)
            )
        }
        EventV1::ServerDelete { id } => {
            if let Some(server) = context.cache.remove_server(&id).await {
                let mut channels = Vec::new();
                let mut voice_states = Vec::new();

                for channel in &server.channels {
                    channels.extend(context.cache.remove_channel(channel).await);
                    voice_states.extend(context.cache.remove_voice_state(channel).await);
                }

                handle_event!(
                    handler,
                    context,
                    server_delete,
                    (server, channels, voice_states)
                )
            }
        }
        EventV1::ServerUpdate { id, data, clear } => {
            if let Some((before, after)) = context
                .cache
                .update_server_with(&id, |server| {
                    let before = server.clone();

                    server.apply_options(data.clone());

                    for field in &clear {
                        match field {
                            FieldsServer::Description => server.description = None,
                            FieldsServer::Categories => server.categories = None,
                            FieldsServer::SystemMessages => server.system_messages = None,
                            FieldsServer::Icon => server.icon = None,
                            FieldsServer::Banner => server.banner = None,
                        }
                    }

                    (before, server.clone())
                })
                .await
            {
                handle_event!(
                    handler,
                    context,
                    server_update,
                    (before, after, data, clear)
                )
            }
        }
        EventV1::ServerMemberJoin { id: _, member, .. } => {
            context.cache.insert_member(member.clone()).await;

            handle_event!(handler, context, server_member_join, (member))
        }
        EventV1::ServerMemberLeave { id, user, reason } => {
            if let Some(member) = context.cache.remove_member(&id, &user).await {
                handle_event!(handler, context, server_member_leave, (member, reason))
            }
        }
        EventV1::ServerMemberUpdate { id, data, clear } => {
            if let Some((before, after)) = context
                .cache
                .update_member_with(&id.server, &id.user, |member| {
                    let before = member.clone();

                    member.apply_options(data.clone());

                    for field in &clear {
                        match field {
                            FieldsMember::Nickname => member.nickname = None,
                            FieldsMember::Avatar => member.avatar = None,
                            FieldsMember::Roles => member.roles.clear(),
                            FieldsMember::Timeout => member.timeout = None,
                            FieldsMember::CanReceive => member.can_publish = true,
                            FieldsMember::CanPublish => member.can_publish = true,
                            FieldsMember::JoinedAt => (),
                        }
                    }

                    (before, member.clone())
                })
                .await
            {
                handle_event!(
                    handler,
                    context,
                    server_member_update,
                    (before, after, data, clear)
                )
            }
        }
        EventV1::ServerRoleUpdate {
            id,
            role_id,
            data,
            clear,
        } => {
            if let Some(Some((before, after))) = context
                .cache
                .update_server_with(&id, |server| {
                    if let Some(role) = server.roles.get_mut(&role_id) {
                        let before = role.clone();

                        role.apply_options(data.clone());

                        for field in &clear {
                            match field {
                                FieldsRole::Colour => role.colour = None,
                            }
                        }

                        Some((before, role.clone()))
                    } else {
                        None
                    }
                })
                .await
            {
                handle_event!(
                    handler,
                    context,
                    server_role_update,
                    (before, after, data, clear)
                )
            }
        }
        EventV1::ServerRoleDelete { id, role_id } => {
            if let Some(Some(role)) = context
                .cache
                .update_server_with(&id, |server| server.roles.remove(&role_id))
                .await
            {
                handle_event!(handler, context, server_role_delete, (role))
            }
        }
        EventV1::ServerRoleRanksUpdate { id, ranks } => {
            if let Some((before, after)) = context
                .cache
                .update_server_with(&id, |server| {
                    let mut before = server.roles.clone().into_values().collect::<Vec<_>>();
                    before.sort_by(|a, b| a.rank.cmp(&b.rank));

                    for (idx, role_id) in ranks.iter().enumerate() {
                        if let Some(role) = server.roles.get_mut(role_id) {
                            role.rank = idx as i64;
                        };
                    }

                    let mut after = server.roles.clone().into_values().collect::<Vec<_>>();
                    after.sort_by(|a, b| a.rank.cmp(&b.rank));

                    (before, after)
                })
                .await
            {
                handle_event!(handler, context, server_role_ranks_update, (before, after))
            }
        }
        EventV1::UserVoiceStateUpdate {
            id,
            channel_id,
            data,
        } => {
            if let Some((before, after)) = context
                .cache
                .update_voice_state_partipant_with(&channel_id, &id, |state| {
                    let before = state.clone();

                    state.apply_options(data.clone());

                    (before, state.clone())
                })
                .await
            {
                handle_event!(
                    handler,
                    context,
                    user_voice_state_update,
                    (before, after, data)
                )
            }
        }
        EventV1::VoiceChannelJoin {
            id,
            state: user_voice_state,
        } => {
            context
                .cache
                .insert_voice_state_partipant(&id, user_voice_state.clone())
                .await;

            handle_event!(
                handler,
                context,
                user_voice_channel_join,
                (id, user_voice_state)
            )
        }
        EventV1::VoiceChannelMove {
            user,
            from,
            to,
            state: user_voice_state,
        } => {
            let before = context
                .cache
                .remove_voice_state_partipant(&from, &user)
                .await;

            context
                .cache
                .insert_voice_state_partipant(&to, user_voice_state.clone())
                .await;

            if let Some(before) = before {
                handle_event!(
                    handler,
                    context,
                    user_voice_channel_move,
                    (user, from, to, before, user_voice_state)
                )
            }
        }
        EventV1::VoiceChannelLeave { id, user } => {
            if let Some(user_voice_state) =
                context.cache.remove_voice_state_partipant(&id, &user).await
            {
                handle_event!(
                    handler,
                    context,
                    user_voice_channel_leave,
                    (id, user_voice_state)
                )
            }
        }
        EventV1::ChannelStartTyping { id, user } => {
            context
                .notifiers
                .invoke_typing_start_waiters(&(id.clone(), user.clone()))
                .await;

            handle_event!(handler, context, typing_start, (id, user))
        }
        EventV1::ChannelStopTyping { id, user } => {
            handle_event!(handler, context, typing_stop, (id, user))
        }
        _ => {}
    }
}

#[async_trait]
#[allow(unused)]
pub trait EventHandler: Sized {
    type Error: From<Error> + Debug + Send + Sync + 'static;

    async fn authenticated(&self, context: Context) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn logout(&self, context: Context) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn pong(&self, context: Context, data: Ping) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn ready(&self, context: Context) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn message(&self, context: Context, message: Message) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn message_update(
        &self,
        context: Context,
        before: Message,
        after: Message,
        partial: PartialMessage,
        clear: Vec<FieldsMessage>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn message_delete(&self, context: Context, message: Message) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn message_react(
        &self,
        context: Context,
        message: Message,
        user_id: String,
        emoji_id: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn message_unreact(
        &self,
        context: Context,
        message: Message,
        user_id: String,
        emoji_id: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn message_remove_reaction(
        &self,
        context: Context,
        message: Message,
        emoji_id: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn message_append(
        &self,
        context: Context,
        message: Message,
        embeds: Vec<Embed>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn user_update(
        &self,
        context: Context,
        before: User,
        after: User,
        partial: PartialUser,
        clear: Vec<FieldsUser>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn bulk_message_delete(
        &self,
        context: Context,
        channel_id: String,
        message_ids: Vec<String>,
        found: Vec<Message>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn channel_create(&self, context: Context, channel: Channel) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn channel_update(
        &self,
        context: Context,
        before: Channel,
        after: Channel,
        partial: PartialChannel,
        clear: Vec<FieldsChannel>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn channel_delete(&self, context: Context, channel: Channel) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn channel_group_user_join(
        &self,
        context: Context,
        channel: Channel,
        user_id: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn channel_group_user_leave(
        &self,
        context: Context,
        channel: Channel,
        user_id: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_create(
        &self,
        context: Context,
        server: Server,
        channels: Vec<Channel>,
        emojis: Vec<Emoji>,
        voice_states: Vec<ChannelVoiceState>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_delete(
        &self,
        context: Context,
        server: Server,
        channels: Vec<Channel>,
        voice_states: Vec<ChannelVoiceState>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_update(
        &self,
        context: Context,
        before: Server,
        after: Server,
        partial: PartialServer,
        clear: Vec<FieldsServer>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn typing_start(
        &self,
        context: Context,
        channel_id: String,
        user_id: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn typing_stop(
        &self,
        context: Context,
        channel_id: String,
        user_id: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_member_join(
        &self,
        context: Context,
        member: Member,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_member_leave(
        &self,
        context: Context,
        member: Member,
        reason: RemovalIntention,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_member_update(
        &self,
        context: Context,
        before: Member,
        after: Member,
        partial: PartialMember,
        clear: Vec<FieldsMember>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_role_update(
        &self,
        context: Context,
        before: Role,
        after: Role,
        partial: PartialRole,
        clear: Vec<FieldsRole>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_role_delete(&self, context: Context, role: Role) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn server_role_ranks_update(
        &self,
        context: Context,
        before: Vec<Role>,
        after: Vec<Role>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn user_voice_state_update(
        &self,
        context: Context,
        before: UserVoiceState,
        after: UserVoiceState,
        partial: PartialUserVoiceState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn user_voice_channel_join(
        &self,
        context: Context,
        channel_id: String,
        user_voice_state: UserVoiceState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn user_voice_channel_move(
        &self,
        context: Context,
        user_id: String,
        from: String,
        to: String,
        before: UserVoiceState,
        after: UserVoiceState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn user_voice_channel_leave(
        &self,
        context: Context,
        user_id: String,
        user_voice_state: UserVoiceState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn error(&self, context: Context, error: Self::Error) {
        log::error!("{error:?}");
    }
}
