use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use stoat_models::v0::{
    Channel, ChannelVoiceState, Emoji, EmojiParent, Member, Message, Server, User, UserVoiceState,
};

use crate::types::StoatConfig;

#[derive(Debug, Clone)]
pub struct GlobalCache {
    pub api_config: Arc<StoatConfig>,

    pub servers: Arc<DashMap<String, Server>>,
    pub users: Arc<DashMap<String, User>>,
    pub members: Arc<DashMap<String, DashMap<String, Member>>>,
    pub channels: Arc<DashMap<String, Channel>>,
    pub messages: Arc<RwLock<VecDeque<Message>>>,
    pub emojis: Arc<DashMap<String, Emoji>>,
    pub voice_states: Arc<DashMap<String, ChannelVoiceState>>,

    #[cfg(feature = "voice")]
    pub voice_connections: Arc<DashMap<String, crate::VoiceConnection>>,

    pub current_user_id: Arc<RwLock<Option<String>>>,
}

impl GlobalCache {
    pub fn new(api_config: StoatConfig) -> Self {
        Self {
            api_config: Arc::new(api_config),
            servers: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
            members: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            messages: Arc::new(RwLock::new(VecDeque::new())),
            emojis: Arc::new(DashMap::new()),
            voice_states: Arc::new(DashMap::new()),

            #[cfg(feature = "voice")]
            voice_connections: Arc::new(DashMap::new()),

            current_user_id: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_server(&self, server_id: &str) -> Option<Server> {
        self.servers.get(server_id).map(|r| r.value().clone())
    }

    pub fn insert_server(&self, server: Server) {
        self.servers.insert(server.id.clone(), server);
    }

    pub fn update_server_with<R>(
        &self,
        server_id: &str,
        f: impl FnOnce(&mut Server) -> R,
    ) -> Option<R> {
        self.servers
            .get_mut(server_id)
            .map(|mut r| f(r.value_mut()))
    }

    pub fn remove_server(&self, server_id: &str) -> Option<Server> {
        self.servers.remove(server_id).map(|(_, server)| server)
    }

    pub fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).map(|r| r.value().clone())
    }

    pub fn insert_user(&self, user: User) {
        self.users.insert(user.id.clone(), user);
    }

    pub fn update_user_with<R>(
        &self,
        user_id: &str,
        f: impl FnOnce(&mut User) -> R,
    ) -> Option<R> {
        self.users.get_mut(user_id).map(|mut r| f(r.value_mut()))
    }

    pub fn remove_user(&self, user_id: &str) -> Option<User> {
        self.users.remove(user_id).map(|(_, server)| server)
    }

    pub fn get_member(&self, server_id: &str, user_id: &str) -> Option<Member> {
        self.members
            .get(server_id)
            .and_then(|members| members.get(user_id).map(|r| r.value().clone()))
    }

    pub fn insert_member(&self, member: Member) {
        self.members
            .entry(member.id.server.clone())
            .or_default()
            .insert(member.id.user.clone(), member);
    }

    pub fn update_member_with<R>(
        &self,
        server_id: &str,
        user_id: &str,
        f: impl FnOnce(&mut Member) -> R,
    ) -> Option<R> {
        self.members
            .get_mut(server_id)
            .and_then(|members| members.get_mut(user_id).map(|mut r| f(r.value_mut())))
    }

    pub fn remove_member(&self, server_id: &str, user_id: &str) -> Option<Member> {
        self.members
            .get_mut(server_id)
            .and_then(|members| members.remove(user_id).map(|(_, member)| member))
    }

    pub fn get_channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.get(channel_id).map(|r| r.value().clone())
    }

    pub fn insert_channel(&self, channel: Channel) {
        self.channels.insert(channel.id().to_string(), channel);
    }

    pub fn update_channel_with<R>(
        &self,
        channel_id: &str,
        f: impl FnOnce(&mut Channel) -> R,
    ) -> Option<R> {
        self.channels
            .get_mut(channel_id)
            .map(|mut r| f(r.value_mut()))
    }

    pub fn remove_channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.remove(channel_id).map(|(_, channel)| channel)
    }

    pub fn get_message(&self, message_id: &str) -> Option<Message> {
        self.messages
            .read()
            .unwrap()
            .iter()
            .find(|msg| &msg.id == message_id)
            .cloned()
    }

    pub fn insert_message(&self, message: Message) {
        let mut messages = self.messages.write().unwrap();

        messages.push_front(message);

        if messages.len() > 1000 {
            messages.pop_back();
        }
    }

    pub fn update_message_with<R>(
        &self,
        message_id: &str,
        f: impl FnOnce(&mut Message) -> R,
    ) -> Option<R> {
        self.messages
            .write()
            .unwrap()
            .iter_mut()
            .find(|msg| &msg.id == message_id)
            .map(f)
    }

    pub fn remove_message(&self, message_id: &str) -> Option<Message> {
        let mut messages = self.messages.write().unwrap();

        if let Some((idx, _)) = messages
            .iter()
            .enumerate()
            .find(|(_, msg)| &msg.id == message_id)
        {
            messages.remove(idx)
        } else {
            None
        }
    }

    pub fn remove_messages(&self, message_ids: &[String]) -> Vec<Message> {
        let mut channel_messages = self.messages.write().unwrap();

        let mut i = 0;
        let end = channel_messages.len();

        let mut messages = Vec::new();

        while i < channel_messages.len() - end {
            if message_ids.contains(&channel_messages[i].id) {
                messages.push(channel_messages.remove(i).unwrap());
            } else {
                i += 1;
            };
        }

        messages
    }

    pub fn get_current_user(&self) -> Option<User> {
        self.users
            .get(&self.get_current_user_id()?)
            .map(|r| r.value().clone())
    }

    pub fn insert_voice_state(&self, voice_state: ChannelVoiceState) {
        self.voice_states
            .insert(voice_state.id.clone(), voice_state);
    }

    pub fn remove_voice_state(&self, channel_id: &str) -> Option<ChannelVoiceState> {
        self.voice_states
            .remove(channel_id)
            .map(|(_, voice_state)| voice_state)
    }

    pub fn get_voice_state(&self, channel_id: &str) -> Option<ChannelVoiceState> {
        self.voice_states.get(channel_id)
            .map(|r| r.value().clone())
    }

    pub fn insert_voice_state_partipant(&self, channel_id: &str, user_voice_state: UserVoiceState) {
        let mut channel_voice_state = self
            .voice_states
            .entry(channel_id.to_string())
            .or_insert_with(|| ChannelVoiceState {
                id: channel_id.to_string(),
                participants: Vec::new(),
            });

        channel_voice_state
            .participants
            .retain(|state| state.id != user_voice_state.id);

        channel_voice_state.participants.push(user_voice_state);
    }

    pub fn remove_voice_state_partipant(
        &self,
        channel_id: &str,
        user_id: &str,
    ) -> Option<UserVoiceState> {
        if let Some(mut channel_voice_state) = self.voice_states.get_mut(channel_id) {
            if let Some((i, _)) = channel_voice_state
                .participants
                .iter()
                .enumerate()
                .find(|(_, state)| &state.id == user_id)
            {
                Some(channel_voice_state.participants.remove(i))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update_voice_state_partipant_with<R>(
        &self,
        channel_id: &str,
        user_id: &str,
        f: impl FnOnce(&mut UserVoiceState) -> R,
    ) -> Option<R> {
        if let Some(mut channel_voice_state) = self.voice_states.get_mut(channel_id) {
            channel_voice_state
                .participants
                .iter_mut()
                .find(|p| p.id == user_id)
                .map(f)
        } else {
            None
        }
    }

    #[cfg(feature = "voice")]
    pub fn insert_voice_connection(&self, connection: crate::VoiceConnection) {
        self.voice_connections
            .insert(connection.channel_id(), connection);
    }

    #[cfg(feature = "voice")]
    pub fn remove_voice_connection(&self, channel_id: &str) -> Option<crate::VoiceConnection> {
        self.voice_connections
            .remove(channel_id)
            .map(|(_, voice_connection)| voice_connection)
    }

    pub fn insert_emoji(&self, emoji: Emoji) {
        self.emojis.insert(emoji.id.clone(), emoji);
    }

    pub fn get_emoji(&self, emoji_id: &str) -> Option<Emoji> {
        self.emojis.get(emoji_id).map(|r| r.value().clone())
    }

    pub fn remove_emoji(&self, emoji_id: &str) -> Option<Emoji> {
        self.emojis.remove(emoji_id).map(|(_, emoji)| emoji)
    }

    pub fn remove_server_emojis(&self, server_id: &str) -> Vec<Emoji> {
        let parent = EmojiParent::Server {
            id: server_id.to_string(),
        };

        let mut emojis = Vec::new();

        // Workaround for no extract_if alternative
        self.emojis.retain(|_, emoji| {
            if &emoji.parent == &parent {
                emojis.push(emoji.clone());

                true
            } else {
                false
            }
        });

        emojis
    }

    pub fn set_current_user_id(&self, user_id: String) {
        *self.current_user_id.write().unwrap() = Some(user_id);
    }

    pub fn get_current_user_id(&self) -> Option<String> {
        self.current_user_id
            .read()
            .unwrap()
            .as_ref()
            .map(|v| v.clone())
    }
}
