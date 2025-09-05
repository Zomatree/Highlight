use std::{fmt::Debug};

use async_trait::async_trait;
use revolt_database::events::client::EventV1;
use revolt_models::v0::{
    Channel, FieldsChannel, FieldsMessage, FieldsUser, Message, RelationshipStatus,
    RemovalIntention,
};

use crate::{cache::GlobalCache, Context};

macro_rules! set_enum_varient_values {
    ($enum: ident, $key: ident, $value: expr, ($($varient: path),+)) => {
        match $enum {
            $($varient { $key, .. } )|+ => { *$key = $value },
            _ => {}
        }
    };
}

macro_rules! update_enum_partial {
    ($value: ident, $data: ident, $key: ident, ($($varient: path),+)) => {
        if let Some(new_value) = $data.$key {
            set_enum_varient_values!($value, $key, new_value, ($($varient),+))
        }
    };

    (optional, $value: ident, $data: ident, $key: ident, ($($varient: path),+)) => {
        set_enum_varient_values!($value, $key, $data.$key, ($($varient),+))
    };
}

macro_rules! update_multi_enum_partial {
    ($value: ident, $data: ident, ($( $( $(@$optional:tt)? optional )? ($key: ident, ($($varient: path),+))),+ $(,)?)) => {
        $(update_enum_partial!($( $($optional)? optional,)? $value, $data, $key, ($($varient),+)));+
    };
}

pub async fn update_state(event: EventV1, state: GlobalCache) {
    match event {
        EventV1::Bulk { v } => {
            for e in v {
                Box::pin(update_state(e, state.clone())).await
            }
        }
        EventV1::Authenticated => {}
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
        } => {
            for user in users.into_iter().flatten() {
                if user.relationship == RelationshipStatus::User {
                    state.current_user_id.write().await.replace(user.id.clone());
                };

                state.insert_user(user).await;
            }

            for server in servers.into_iter().flatten() {
                state.insert_server(server).await;
            }

            for channel in channels.into_iter().flatten() {
                state.insert_channel(channel).await;
            }

            for member in members.into_iter().flatten() {
                state.insert_member(member).await;
            }
        }
        EventV1::Message(message) => {
            state.insert_message(message).await;
        }
        EventV1::MessageUpdate {
            id,
            channel: _,
            data,
            clear,
        } => {
            state.update_message_with(&id, |message| {
                message.apply_options(data);

                for field in clear {
                    match field {
                        FieldsMessage::Pinned => message.pinned = None,
                    }
                }
            }).await
        }
        EventV1::UserUpdate {
            id, data, clear, ..
        } => {
            state.update_user_with(&id, |user| {
                user.apply_options(data);

                for field in clear {
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
            }).await
        }
        EventV1::BulkMessageDelete { channel: _, ids } => {
            state.messages.write().await.retain(|m| !ids.contains(&m.id));
        }
        EventV1::ChannelAck { .. } => {}
        EventV1::ChannelCreate(channel) => {
            match &channel {
                Channel::TextChannel { id, server, .. }
                | Channel::VoiceChannel { id, server, .. } => {
                    state.update_server_with(&server, |server| server.channels.push(id.to_string())).await
                }
                _ => {}
            };

            state.insert_channel(channel).await;
        }
        EventV1::ChannelDelete { id } => {
            if let Some(channel) = state.remove_channel(&id).await {
                match &channel {
                    Channel::TextChannel { id, server, .. }
                    | Channel::VoiceChannel { id, server, .. } => {
                        state.update_server_with(&server, |server| server.channels.retain(|c_id| c_id != id)).await
                    }
                    _ => {}
                };
            }
        }
        EventV1::ChannelGroupJoin { id, user } => {
            state.update_channel_with(&id, |channel| {
                if let Channel::Group { recipients, .. } = channel {
                    recipients.push(user)
                }
            }).await
        }
        EventV1::ChannelGroupLeave { id, user } => {
            state.update_channel_with(&id, |channel| {
                if let Channel::Group { recipients, .. } = channel {
                    recipients.retain(|u_id| u_id != &user)
                }
            }).await
        }
        EventV1::ChannelUpdate { id, data, clear } => {
            state.update_channel_with(&id, |channel| {
                update_multi_enum_partial!(
                    channel,
                    data,
                    (
                        (name, (Channel::TextChannel, Channel::VoiceChannel)),
                        (owner, (Channel::Group)),
                        optional(
                            description,
                            (Channel::Group, Channel::TextChannel, Channel::VoiceChannel)
                        ),
                        optional(
                            icon,
                            (Channel::Group, Channel::TextChannel, Channel::VoiceChannel)
                        ),
                        (
                            nsfw,
                            (Channel::Group, Channel::TextChannel, Channel::VoiceChannel)
                        ),
                        (active, (Channel::DirectMessage)),
                        optional(permissions, (Channel::Group)),
                        (
                            role_permissions,
                            (Channel::TextChannel, Channel::VoiceChannel)
                        ),
                        optional(
                            default_permissions,
                            (Channel::TextChannel, Channel::VoiceChannel)
                        ),
                        optional(
                            last_message_id,
                            (Channel::DirectMessage, Channel::Group, Channel::TextChannel)
                        )
                    )
                );

                for field in clear {
                    match field {
                        FieldsChannel::Description => set_enum_varient_values!(
                            channel,
                            description,
                            None,
                            (Channel::Group, Channel::TextChannel, Channel::VoiceChannel)
                        ),
                        FieldsChannel::Icon => set_enum_varient_values!(
                            channel,
                            icon,
                            None,
                            (Channel::Group, Channel::TextChannel, Channel::VoiceChannel)
                        ),
                        FieldsChannel::DefaultPermissions => set_enum_varient_values!(
                            channel,
                            default_permissions,
                            None,
                            (Channel::TextChannel, Channel::VoiceChannel)
                        ),
                    }
                }
            }).await
        }
        EventV1::MessageAppend {
            id,
            channel: _,
            append,
        } => {
            state.update_message_with(&id, |message| {
                if let Some(embeds) = append.embeds {
                    message.embeds.get_or_insert_default().extend(embeds);
                }
            }).await
        }
        EventV1::ServerCreate {
            id,
            server,
            channels,
            emojis: _,
        } => {
            state.insert_server(server).await;

            for channel in channels {
                state.insert_channel(channel).await
            }
        }
        EventV1::ServerDelete { id } => {
            if let Some(server) = state.remove_server(&id).await {
                for channel in server.channels {
                    state.remove_channel(&channel).await;
                }
            }
        }
        EventV1::ServerMemberJoin { id, user } => {
            // TODO insert member when update is out
        }
        event => {
            println!("Unhandled Event {:?}", event);
        }
    }
}

#[async_trait]
#[allow(unused)]
pub trait EventHandler<E: Debug + Send + Sync + 'static>: Sized {
    async fn authenticated(&self, context: Context) -> Result<(), E> {
        Ok(())
    }

    async fn ready(&self, context: Context) -> Result<(), E> {
        Ok(())
    }

    async fn message(&self, context: Context, message: Message) -> Result<(), E> {
        Ok(())
    }

    async fn start_typing(
        &self,
        context: Context,
        channel_id: String,
        user_id: String,
    ) -> Result<(), E> {
        Ok(())
    }

    async fn stop_typing(
        &self,
        context: Context,
        channel_id: String,
        user_id: String,
    ) -> Result<(), E> {
        Ok(())
    }

    async fn server_member_join(
        &self,
        context: Context,
        server_id: String,
        user_id: String,
    ) -> Result<(), E> {
        Ok(())
    }

    async fn server_member_leave(
        &self,
        context: Context,
        server_id: String,
        user_id: String,
        reason: RemovalIntention,
    ) -> Result<(), E> {
        Ok(())
    }

    async fn error(&self, context: Context, error: E) {
        println!("Error: {error:?}");
    }
}
