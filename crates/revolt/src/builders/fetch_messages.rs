use reqwest::Method;
use revolt_models::v0::{
    BulkMessageResponse, Member, Message, MessageSort, OptionsQueryMessages, User,
};

use crate::{HttpClient, error::Error};

#[derive(Debug, Clone)]
pub struct MessagesWithUsers {
    pub messages: Vec<Message>,
    pub users: Vec<User>,
    pub members: Vec<Member>,
}

pub struct FetchMessagesBuilder<'a> {
    http: &'a HttpClient,
    channel_id: &'a str,
    data: OptionsQueryMessages,
}

impl<'a> FetchMessagesBuilder<'a> {
    pub fn new(http: &'a HttpClient, channel_id: &'a str) -> Self {
        Self {
            http,
            channel_id,
            data: OptionsQueryMessages {
                limit: None,
                before: None,
                after: None,
                sort: None,
                nearby: None,
                include_users: None,
            },
        }
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.data.limit = Some(limit);

        self
    }

    pub fn before(mut self, before: String) -> Self {
        self.data.before = Some(before);

        self
    }

    pub fn after(mut self, after: String) -> Self {
        self.data.after = Some(after);

        self
    }

    pub fn sort(mut self, sort: MessageSort) -> Self {
        self.data.sort = Some(sort);

        self
    }

    pub fn nearby(mut self, nearby: String) -> Self {
        self.data.nearby = Some(nearby);

        self
    }

    pub async fn build_raw(mut self, with_users: bool) -> Result<BulkMessageResponse, Error> {
        self.data.include_users = Some(with_users);

        self.http
            .request(
                Method::GET,
                format!("/channels/{}/messages", &self.channel_id),
            )
            .query(&self.data)
            .response()
            .await
    }

    pub async fn build_with_users(self) -> Result<MessagesWithUsers, Error> {
        let bulk = self.build_raw(true).await?;

        if let BulkMessageResponse::MessagesAndUsers {
            messages,
            users,
            members,
        } = bulk
        {
            Ok(MessagesWithUsers {
                messages,
                users,
                members: members.unwrap_or_default(),
            })
        } else {
            Err(Error::InternalError)
        }
    }

    pub async fn build(self) -> Result<Vec<Message>, Error> {
        let bulk = self.build_raw(false).await?;

        if let BulkMessageResponse::JustMessages(messages) = bulk {
            Ok(messages)
        } else {
            Err(Error::InternalError)
        }
    }
}
