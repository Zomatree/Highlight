use std::collections::{HashMap, VecDeque};

use revolt_models::v0::{Channel, Member, Message, Server, User};

use crate::http::RevoltConfig;

#[derive(Debug, Clone)]
pub struct GlobalState {
    pub api_config: RevoltConfig,

    pub servers: HashMap<String, Server>,
    pub users: HashMap<String, User>,
    pub members: HashMap<String, HashMap<String, Member>>,
    pub channels: HashMap<String, Channel>,
    pub messages: VecDeque<Message>,

    pub current_user: Option<User>,
}

impl GlobalState {
    pub fn new(api_config: RevoltConfig) -> Self {
        Self {
            api_config,
            servers: HashMap::new(),
            users: HashMap::new(),
            members: HashMap::new(),
            channels: HashMap::new(),
            messages: VecDeque::new(),
            current_user: None,
        }
    }
}
