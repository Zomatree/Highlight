use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use revolt_database::events::client::EventV1;
use revolt_models::v0::{
    Channel, FieldsChannel, FieldsMessage, FieldsUser, Message, RelationshipStatus,
};

use crate::{http::HttpClient, state::GlobalState};

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

pub fn update_state(event: EventV1, state: &mut GlobalState) {
    match event {
        EventV1::Bulk { v } => {
            for e in v {
                update_state(e, state)
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
                    state.current_user = Some(user.clone());
                };

                state.users.insert(user.id.clone(), user);
            }

            for server in servers.into_iter().flatten() {
                state.members.insert(server.id.clone(), HashMap::new());
                state.servers.insert(server.id.clone(), server);
            }

            for channel in channels.into_iter().flatten() {
                state.channels.insert(channel.id().to_string(), channel);
            }

            for member in members.into_iter().flatten() {
                state
                    .members
                    .get_mut(&member.id.server)
                    .map(|members| members.insert(member.id.user.clone(), member));
            }
        }
        EventV1::Message(mut message) => {
            if let Some(user) = message.user.take() {
                state.users.insert(user.id.clone(), user);
            };

            if let Some(member) = message.member.take() {
                state
                    .members
                    .get_mut(&member.id.server)
                    .map(|members| members.insert(member.id.user.clone(), member));
            };

            state.messages.push_front(message);

            if state.messages.len() > 1000 {
                state.messages.pop_back();
            }
        }
        EventV1::MessageUpdate {
            id,
            channel: _,
            data,
            clear,
        } => {
            if let Some(message) = state.messages.iter_mut().find(|m| m.id == id) {
                message.apply_options(data);

                for field in clear {
                    match field {
                        FieldsMessage::Pinned => message.pinned = None,
                    }
                }
            }
        }
        EventV1::UserUpdate {
            id, data, clear, ..
        } => {
            if let Some(user) = state.users.get_mut(&id) {
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
            }
        }
        EventV1::BulkMessageDelete { channel: _, ids } => {
            state.messages.retain(|m| !ids.contains(&m.id));
        }
        EventV1::ChannelAck { .. } => {}
        EventV1::ChannelCreate(channel) => {
            match &channel {
                Channel::TextChannel { id, server, .. }
                | Channel::VoiceChannel { id, server, .. } => {
                    state
                        .servers
                        .get_mut(server)
                        .unwrap()
                        .channels
                        .push(id.to_string());
                }
                _ => {}
            };

            state.channels.insert(channel.id().to_string(), channel);
        }
        EventV1::ChannelDelete { id } => {
            if let Some(channel) = state.channels.remove(&id) {
                match &channel {
                    Channel::TextChannel { id, server, .. }
                    | Channel::VoiceChannel { id, server, .. } => {
                        let server = state.servers.get_mut(server).unwrap();

                        server.channels.retain(|c_id| c_id != id)
                    }
                    _ => {}
                };
            }
        }
        EventV1::ChannelGroupJoin { id, user } => {
            if let Some(channel) = state.channels.get_mut(&id) {
                if let Channel::Group { recipients, .. } = channel {
                    recipients.push(user)
                }
            }
        }
        EventV1::ChannelGroupLeave { id, user } => {
            if let Some(channel) = state.channels.get_mut(&id) {
                if let Channel::Group { recipients, .. } = channel {
                    recipients.retain(|u_id| u_id != &user)
                }
            }
        }
        EventV1::ChannelUpdate { id, data, clear } => {
            if let Some(channel) = state.channels.get_mut(&id) {
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
            }
        }
        EventV1::MessageAppend {
            id,
            channel: _,
            append,
        } => {
            if let Some(message) = state.messages.iter_mut().find(|m| m.id == id) {
                if let Some(embeds) = append.embeds {
                    message.embeds.get_or_insert_default().extend(embeds);
                }
            }
        }
        EventV1::ServerCreate {
            id,
            server,
            channels,
            emojis: _,
        } => {
            state.servers.insert(id, server);

            for channel in channels {
                state.channels.insert(channel.id().to_string(), channel);
            }
        }
        EventV1::ServerDelete { id } => {
            if let Some(server) = state.servers.remove(&id) {
                for channel in server.channels {
                    state.channels.remove(&channel);
                }
            }
        }
        event => {
            println!("Unhandled Event {:?}", event);
        }
    }
}

#[derive(Debug)]
pub struct Context<'a> {
    pub state: &'a mut GlobalState,
    pub http: HttpClient,
}

#[async_trait]
#[allow(unused)]
pub trait EventHandler<E: Debug + Send + Sync + 'static>: Sized {
    async fn authenticated(&self, context: &Context<'_>) -> Result<(), E> {
        Ok(())
    }
    async fn message(&self, context: &Context<'_>, message: Message) -> Result<(), E> {
        Ok(())
    }

    async fn error(&self, context: &Context<'_>, error: E) {
        println!("Error: {error:?}");
    }
}
