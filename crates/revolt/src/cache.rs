use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use revolt_models::v0::{Channel, Member, Message, Server, User};
use tokio::sync::RwLock;

use crate::http::RevoltConfig;

#[derive(Debug, Clone)]
pub struct GlobalCache {
    pub api_config: Arc<RevoltConfig>,

    pub servers: Arc<RwLock<HashMap<String, Server>>>,
    pub users: Arc<RwLock<HashMap<String, User>>>,
    pub members: Arc<RwLock<HashMap<String, HashMap<String, Member>>>>,
    pub channels: Arc<RwLock<HashMap<String, Channel>>>,
    pub messages: Arc<RwLock<VecDeque<Message>>>,

    pub current_user_id: Arc<RwLock<Option<String>>>,
}

impl GlobalCache {
    pub fn new(api_config: RevoltConfig) -> Self {
        Self {
            api_config: Arc::new(api_config),
            servers: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(VecDeque::new())),
            current_user_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_server(&self, server_id: &str) -> Option<Server> {
        self.servers.read().await.get(server_id).cloned()
    }

    pub async fn insert_server(&self, server: Server) {
        self.servers.write().await.insert(server.id.clone(), server);
    }

    pub async fn update_server_with(&self, server_id: &str, f: impl FnOnce(&mut Server)) {
        self.servers.write().await.get_mut(server_id).map(f);
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

    pub async fn update_user_with(&self, user_id: &str, f: impl FnOnce(&mut User)) {
        self.users.write().await.get_mut(user_id).map(f);
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

    pub async fn update_member_with(
        &self,
        server_id: &str,
        user_id: &str,
        f: impl FnOnce(&mut Member),
    ) {
        self.members
            .write()
            .await
            .get_mut(server_id)
            .and_then(|members| members.get_mut(user_id))
            .map(f);
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

    pub async fn update_channel_with(&self, channel_id: &str, f: impl FnOnce(&mut Channel)) {
        self.channels.write().await.get_mut(channel_id).map(f);
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

    pub async fn update_message_with(&self, message_id: &str, f: impl FnOnce(&mut Message)) {
        self.messages
            .write()
            .await
            .iter_mut()
            .find(|msg| &msg.id == message_id)
            .map(f);
    }

    pub async fn get_current_user(&self) -> Option<User> {
        let guard = self.current_user_id.read().await;

        self.users.read().await.get(guard.as_ref()?).cloned()
    }
}
