use std::borrow::Cow;

use async_trait::async_trait;
use stoat_models::v0::{Channel, Member, Server, User};
use stoat_permissions::{
    ChannelType, DEFAULT_PERMISSION_DIRECT_MESSAGE, Override, RelationshipStatus,
};

use crate::{GlobalCache, HttpClient};

pub use stoat_permissions::{
    ChannelPermission, UserPermission, calculate_channel_permissions, calculate_server_permissions,
    calculate_user_permissions,
};

pub struct PermissionQuery<'a> {
    cache: GlobalCache,
    http: HttpClient,

    perspective: Cow<'a, User>,
    user: Option<Cow<'a, User>>,
    channel: Option<Cow<'a, Channel>>,
    server: Option<Cow<'a, Server>>,
    member: Option<Cow<'a, Member>>,
}

impl<'a> PermissionQuery<'a> {
    pub fn new(cache: GlobalCache, http: HttpClient, perspective: Cow<'a, User>) -> Self {
        Self {
            cache,
            http,
            perspective,
            user: None,
            channel: None,
            server: None,
            member: None,
        }
    }

    /// Use user
    pub fn user(mut self, user: Cow<'a, User>) -> Self {
        self.user = Some(user);

        self
    }

    /// Use channel
    pub fn channel(mut self, channel: Cow<'a, Channel>) -> Self {
        self.channel = Some(channel);

        self
    }

    /// Use server
    pub fn server(mut self, server: Cow<'a, Server>) -> Self {
        self.server = Some(server);

        self
    }

    /// Use member
    pub fn member(mut self, member: Cow<'a, Member>) -> Self {
        self.member = Some(member);

        self
    }
}

#[async_trait]
impl stoat_permissions::PermissionQuery for PermissionQuery<'_> {
    async fn are_we_privileged(&mut self) -> bool {
        self.perspective.privileged
    }

    /// Is our perspective user a bot?
    async fn are_we_a_bot(&mut self) -> bool {
        self.perspective.bot.is_some()
    }

    /// Is our perspective user and the currently selected user the same?
    async fn are_the_users_same(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            self.perspective.id == other_user.id
        } else {
            false
        }
    }

    /// Get the relationship with have with the currently selected user
    async fn user_relationship(&mut self) -> RelationshipStatus {
        if let Some(other_user) = &self.user {
            if self.perspective.id == other_user.id {
                return RelationshipStatus::User;
            } else if let Some(bot) = &other_user.bot {
                if self.perspective.id == bot.owner_id {
                    return RelationshipStatus::User;
                }
            }

            for entry in &self.perspective.relations {
                if entry.user_id == other_user.id {
                    return match entry.status {
                        stoat_models::v0::RelationshipStatus::None => RelationshipStatus::None,
                        stoat_models::v0::RelationshipStatus::User => RelationshipStatus::User,
                        stoat_models::v0::RelationshipStatus::Friend => RelationshipStatus::Friend,
                        stoat_models::v0::RelationshipStatus::Outgoing => {
                            RelationshipStatus::Outgoing
                        }
                        stoat_models::v0::RelationshipStatus::Incoming => {
                            RelationshipStatus::Incoming
                        }
                        stoat_models::v0::RelationshipStatus::Blocked => {
                            RelationshipStatus::Blocked
                        }
                        stoat_models::v0::RelationshipStatus::BlockedOther => {
                            RelationshipStatus::BlockedOther
                        }
                    };
                }
            }
        }

        RelationshipStatus::None
    }

    /// Whether the currently selected user is a bot
    async fn user_is_bot(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            other_user.bot.is_some()
        } else {
            false
        }
    }

    async fn have_mutual_connection(&mut self) -> bool {
        true
    }

    // * For calculating server permission

    /// Is our perspective user the server's owner?
    async fn are_we_server_owner(&mut self) -> bool {
        if let Some(server) = &self.server {
            server.owner == self.perspective.id
        } else {
            false
        }
    }

    /// Is our perspective user a member of the server?
    async fn are_we_a_member(&mut self) -> bool {
        if let Some(server) = &self.server {
            if self.member.is_some() {
                true
            } else if let Some(member) = self.cache.get_member(&server.id, &self.perspective.id) {
                self.member = Some(Cow::Owned(member.clone()));

                true
            } else if let Ok(member) = self
                .http
                .fetch_member(&server.id, &self.perspective.id)
                .await
            {
                self.member = Some(Cow::Owned(member));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get default server permission
    async fn get_default_server_permissions(&mut self) -> u64 {
        if let Some(server) = &self.server {
            server.default_permissions as u64
        } else {
            0
        }
    }

    /// Get the ordered role overrides (from lowest to highest) for this member in this server
    async fn get_our_server_role_overrides(&mut self) -> Vec<Override> {
        if let Some(server) = &self.server {
            let member_roles = self
                .member
                .as_ref()
                .map(|member| member.roles.clone())
                .unwrap_or_default();

            let mut roles = server
                .roles
                .iter()
                .filter(|(id, _)| member_roles.contains(id))
                .map(|(_, role)| {
                    let v: Override = role.permissions.into();
                    (role.rank, v)
                })
                .collect::<Vec<(i64, Override)>>();

            roles.sort_by(|a, b| b.0.cmp(&a.0));
            roles.into_iter().map(|(_, v)| v).collect()
        } else {
            vec![]
        }
    }

    /// Is our perspective user timed out on this server?
    async fn are_we_timed_out(&mut self) -> bool {
        if let Some(member) = &self.member {
            member.timeout.is_some()
        } else {
            false
        }
    }

    /// Is the member muted?
    async fn do_we_have_publish_overwrites(&mut self) -> bool {
        self.member.as_ref().is_none_or(|member| member.can_publish)
    }

    /// Is the member deafend?
    async fn do_we_have_receive_overwrites(&mut self) -> bool {
        self.member.as_ref().is_none_or(|member| member.can_receive)
    }

    // * For calculating channel permission

    /// Get the type of the channel
    async fn get_channel_type(&mut self) -> ChannelType {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::DirectMessage { .. })
                | Cow::Owned(Channel::DirectMessage { .. }) => ChannelType::DirectMessage,
                Cow::Borrowed(Channel::Group { .. }) | Cow::Owned(Channel::Group { .. }) => {
                    ChannelType::Group
                }
                Cow::Borrowed(Channel::SavedMessages { .. })
                | Cow::Owned(Channel::SavedMessages { .. }) => ChannelType::SavedMessages,
                Cow::Borrowed(Channel::TextChannel { .. })
                | Cow::Owned(Channel::TextChannel { .. }) => ChannelType::ServerChannel,
            }
        } else {
            ChannelType::Unknown
        }
    }

    /// Get the default channel permissions
    /// Group channel defaults should be mapped to an allow-only override
    async fn get_default_channel_permissions(&mut self) -> Override {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::Group { permissions, .. })
                | Cow::Owned(Channel::Group { permissions, .. }) => Override {
                    allow: permissions.unwrap_or(*DEFAULT_PERMISSION_DIRECT_MESSAGE as i64) as u64,
                    deny: 0,
                },
                Cow::Borrowed(Channel::TextChannel {
                    default_permissions,
                    ..
                })
                | Cow::Owned(Channel::TextChannel {
                    default_permissions,
                    ..
                }) => default_permissions.unwrap_or_default().into(),
                _ => Default::default(),
            }
        } else {
            Default::default()
        }
    }

    /// Get the ordered role overrides (from lowest to highest) for this member in this channel
    async fn get_our_channel_role_overrides(&mut self) -> Vec<Override> {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::TextChannel {
                    role_permissions, ..
                })
                | Cow::Owned(Channel::TextChannel {
                    role_permissions, ..
                }) => {
                    if let Some(server) = &self.server {
                        let member_roles = self
                            .member
                            .as_ref()
                            .map(|member| member.roles.clone())
                            .unwrap_or_default();

                        let mut roles = role_permissions
                            .iter()
                            .filter(|(id, _)| member_roles.contains(id))
                            .filter_map(|(id, permission)| {
                                server.roles.get(id).map(|role| {
                                    let v: Override = (*permission).into();
                                    (role.rank, v)
                                })
                            })
                            .collect::<Vec<(i64, Override)>>();

                        roles.sort_by(|a, b| b.0.cmp(&a.0));
                        roles.into_iter().map(|(_, v)| v).collect()
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Do we own this group or saved messages channel if it is one of those?
    async fn do_we_own_the_channel(&mut self) -> bool {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::Group { owner, .. })
                | Cow::Owned(Channel::Group { owner, .. }) => owner == &self.perspective.id,
                Cow::Borrowed(Channel::SavedMessages { user, .. })
                | Cow::Owned(Channel::SavedMessages { user, .. }) => user == &self.perspective.id,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Are we a recipient of this channel?
    async fn are_we_part_of_the_channel(&mut self) -> bool {
        if let Some(
            Cow::Borrowed(Channel::DirectMessage { recipients, .. })
            | Cow::Owned(Channel::DirectMessage { recipients, .. })
            | Cow::Borrowed(Channel::Group { recipients, .. })
            | Cow::Owned(Channel::Group { recipients, .. }),
        ) = &self.channel
        {
            recipients.contains(&self.perspective.id)
        } else {
            false
        }
    }

    /// Set the current user as the recipient of this channel
    /// (this will only ever be called for DirectMessage channels, use unimplemented!() for other code paths)
    async fn set_recipient_as_user(&mut self) {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::DirectMessage { recipients, .. })
                | Cow::Owned(Channel::DirectMessage { recipients, .. }) => {
                    let recipient_id = recipients
                        .iter()
                        .find(|recipient| recipient != &&self.perspective.id)
                        .expect("Missing recipient for DM");

                    if let Some(user) = self.cache.get_user(recipient_id) {
                        self.user.replace(Cow::Owned(user.clone()));
                    } else if let Ok(user) = self.http.fetch_user(recipient_id).await {
                        self.user.replace(Cow::Owned(user));
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    /// Set the current server as the server owning this channel
    /// (this will only ever be called for server channels, use unimplemented!() for other code paths)
    async fn set_server_from_channel(&mut self) {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::TextChannel { server, .. })
                | Cow::Owned(Channel::TextChannel { server, .. }) => {
                    if let Some(known_server) = self.server.as_ref().map(|server| server.as_ref()) {
                        if server == &known_server.id {
                            // Already cached, return early.
                            return;
                        }
                    }

                    if let Some(server) = self.cache.get_server(server) {
                        self.server.replace(Cow::Owned(server.clone()));
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub fn user_permissions_query<'a>(
    cache: GlobalCache,
    http: HttpClient,
    user: Cow<'a, User>,
) -> PermissionQuery<'a> {
    let ourself = cache.get_current_user().unwrap();

    PermissionQuery::new(cache, http, Cow::Owned(ourself)).user(user)
}
