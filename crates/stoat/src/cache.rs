use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use stoat_models::v0::{Channel, ChannelVoiceState, Member, Message, Server, User, UserVoiceState};
use tokio::sync::RwLock;

use crate::http::StoatConfig;

#[derive(Debug, Clone)]
pub struct GlobalCache {
    pub api_config: Arc<StoatConfig>,

    pub servers: Arc<RwLock<HashMap<String, Server>>>,
    pub users: Arc<RwLock<HashMap<String, User>>>,
    pub members: Arc<RwLock<HashMap<String, HashMap<String, Member>>>>,
    pub channels: Arc<RwLock<HashMap<String, Channel>>>,
    pub messages: Arc<RwLock<VecDeque<Message>>>,
    pub voice_states: Arc<RwLock<HashMap<String, ChannelVoiceState>>>,

    #[cfg(feature = "voice")]
    pub voice_connections: Arc<RwLock<HashMap<String, crate::VoiceConnection>>>,

    pub current_user_id: Arc<RwLock<Option<String>>>,
}

impl GlobalCache {
    pub fn new(api_config: StoatConfig) -> Self {
        Self {
            api_config: Arc::new(api_config),
            servers: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(VecDeque::new())),
            voice_states: Arc::new(RwLock::new(HashMap::new())),

            #[cfg(feature = "voice")]
            voice_connections: Arc::new(RwLock::new(HashMap::new())),

            current_user_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_server(&self, server_id: &str) -> Option<Server> {
        self.servers.read().await.get(server_id).cloned()
    }

    pub async fn insert_server(&self, server: Server) {
        self.servers.write().await.insert(server.id.clone(), server);
    }

    pub async fn update_server_with<R>(&self, server_id: &str, f: impl FnOnce(&mut Server) -> R) -> Option<R> {
        self.servers.write().await.get_mut(server_id).map(f)
    }

    pub async fn remove_server(&self, server_id: &str) -> Option<Server> {
        self.servers.write().await.remove(server_id)
    }

    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.read().await.get(user_id).cloned()
    }

    pub async fn insert_user(&self, user: User) {
        self.users.write().await.insert(user.id.clone(), user);
    }

    pub async fn update_user_with<R>(&self, user_id: &str, f: impl FnOnce(&mut User) -> R) -> Option<R> {
        self.users.write().await.get_mut(user_id).map(f)
    }

    pub async fn remove_user(&self, user_id: &str) -> Option<User> {
        self.users.write().await.remove(user_id)
    }

    pub async fn get_member(&self, server_id: &str, user_id: &str) -> Option<Member> {
        self.members
            .read()
            .await
            .get(server_id)
            .and_then(|members| members.get(user_id))
            .cloned()
    }

    pub async fn insert_member(&self, member: Member) {
        self.members
            .write()
            .await
            .entry(member.id.server.clone())
            .or_default()
            .insert(member.id.user.clone(), member);
    }

    pub async fn update_member_with<R>(
        &self,
        server_id: &str,
        user_id: &str,
        f: impl FnOnce(&mut Member) -> R,
    ) -> Option<R> {
        self.members
            .write()
            .await
            .get_mut(server_id)
            .and_then(|members| members.get_mut(user_id))
            .map(f)
    }

    pub async fn remove_member(&self, server_id: &str, user_id: &str) -> Option<Member> {
        self.members
            .write()
            .await
            .get_mut(server_id)
            .and_then(|members| members.remove(user_id))
    }

    pub async fn get_channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.read().await.get(channel_id).cloned()
    }

    pub async fn insert_channel(&self, channel: Channel) {
        self.channels
            .write()
            .await
            .insert(channel.id().to_string(), channel);
    }

    pub async fn update_channel_with<R>(&self, channel_id: &str, f: impl FnOnce(&mut Channel) -> R) -> Option<R> {
        self.channels.write().await.get_mut(channel_id).map(f)
    }

    pub async fn remove_channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.write().await.remove(channel_id)
    }

    pub async fn get_message(&self, message_id: &str) -> Option<Message> {
        self.messages
            .read()
            .await
            .iter()
            .find(|msg| &msg.id == message_id)
            .cloned()
    }

    pub async fn insert_message(&self, message: Message) {
        let mut messages = self.messages.write().await;

        messages.push_front(message);

        if messages.len() > 1000 {
            messages.pop_back();
        }
    }

    pub async fn update_message_with<R>(
        &self,
        message_id: &str,
        f: impl FnOnce(&mut Message) -> R,
    ) -> Option<R> {
        self.messages
            .write()
            .await
            .iter_mut()
            .find(|msg| &msg.id == message_id)
            .map(f)
    }

    pub async fn remove_message(&self, message_id: &str) -> Option<Message> {
        let mut messages = self.messages.write().await;

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

    pub async fn get_current_user(&self) -> Option<User> {
        let guard = self.current_user_id.read().await;

        self.users.read().await.get(guard.as_ref()?).cloned()
    }

    pub async fn insert_voice_state(&self, voice_state: ChannelVoiceState) {
        self.voice_states
            .write()
            .await
            .insert(voice_state.id.clone(), voice_state);
    }

    pub async fn remove_voice_state(&self, channel_id: &str) -> Option<ChannelVoiceState> {
        self.voice_states.write().await.remove(channel_id)
    }

    pub async fn insert_voice_state_partipant(
        &self,
        channel_id: &str,
        user_voice_state: UserVoiceState,
    ) {
        let mut lock = self.voice_states.write().await;

        let channel_voice_state =
            lock.entry(channel_id.to_string())
                .or_insert_with(|| ChannelVoiceState {
                    id: channel_id.to_string(),
                    participants: Vec::new(),
                });

        channel_voice_state
            .participants
            .retain(|state| state.id != user_voice_state.id);
        channel_voice_state.participants.push(user_voice_state);
    }

    pub async fn remove_voice_state_partipant(&self, channel_id: &str, user_id: &str) -> Option<UserVoiceState> {
        if let Some(channel_voice_state) = self.voice_states.write().await.get_mut(channel_id) {
            if let Some((i, _)) = channel_voice_state
                .participants
                .iter()
                .enumerate()
                .find(|(_, state)| &state.id == user_id) {
                    Some(channel_voice_state
                        .participants
                        .remove(i))
                } else {
                    None
                }
        } else {
            None
        }
    }

    pub async fn update_voice_state_partipant_with<R>(
        &self,
        channel_id: &str,
        user_id: &str,
        f: impl FnOnce(&mut UserVoiceState) -> R,
    ) -> Option<R> {
        if let Some(channel_voice_state) = self.voice_states.write().await.get_mut(channel_id) {
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
    pub async fn insert_voice_connection(&self, connection: crate::VoiceConnection) {
        self.voice_connections
            .write()
            .await
            .insert(connection.channel_id(), connection);
    }

    #[cfg(feature = "voice")]
    pub async fn remove_voice_connection(
        &self,
        channel_id: &str,
    ) -> Option<crate::VoiceConnection> {
        self.voice_connections.write().await.remove(channel_id)
    }
}
