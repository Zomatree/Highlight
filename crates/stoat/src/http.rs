use reqwest::{Client, Method, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use stoat_models::v0::{
    BulkMessageResponse, Channel, CreateVoiceUserResponse, DataEditMessage, DataJoinCall,
    DataMessageSend, Member, Message, OptionsQueryMessages, User,
};

use crate::error::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct CaptchaFeature {
    pub enabled: bool,
    pub key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Feature {
    pub enabled: bool,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VoiceNode {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub public_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VoiceFeature {
    pub enabled: bool,
    pub nodes: Vec<VoiceNode>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StoatFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Feature,
    pub january: Feature,
    pub livekit: VoiceFeature,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BuildInformation {
    pub commit_sha: String,
    pub commit_timestamp: String,
    pub semver: String,
    pub origin_url: String,
    pub timestamp: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StoatConfig {
    pub revolt: String,
    pub features: StoatFeatures,
    pub ws: String,
    pub app: String,
    pub vapid: String,
    pub build: BuildInformation,
}

#[derive(Clone, Debug)]
pub struct HttpClient {
    pub base: String,
    pub token: Option<String>,
    pub inner: Client,
}

impl HttpClient {
    pub fn new(base: String, token: Option<String>) -> Self {
        HttpClient {
            base,
            token,
            inner: Client::new(),
        }
    }

    pub fn request(&self, method: Method, route: impl AsRef<str>) -> HttpRequest {
        let mut builder = self
            .inner
            .request(method, format!("{}{}", &self.base, route.as_ref()))
            .header("Accept", "application/json");

        if let Some(token) = &self.token {
            builder = builder.header("x-bot-token", token);
        }

        HttpRequest { builder }
    }

    pub async fn get_root(&self) -> Result<StoatConfig, Error> {
        self.request(Method::GET, "/").response().await
    }

    pub async fn send_message(
        &self,
        channel_id: &str,
        data: &DataMessageSend,
    ) -> Result<Message, Error> {
        self.request(Method::POST, format!("/channels/{}/messages", channel_id))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_user(&self, user_id: &str) -> Result<User, Error> {
        self.request(Method::GET, format!("/users/{user_id}"))
            .response()
            .await
    }

    pub async fn fetch_messages<'a>(
        &self,
        channel_id: &str,
        data: &OptionsQueryMessages,
    ) -> Result<BulkMessageResponse, Error> {
        self.request(Method::GET, format!("/channels/{}/messages", channel_id))
            .query(data)
            .response()
            .await
    }

    pub async fn open_dm(&self, user_id: &str) -> Result<Channel, Error> {
        self.request(Method::GET, format!("/users/{user_id}/dm"))
            .response()
            .await
    }

    pub async fn fetch_member(&self, server_id: &str, user_id: &str) -> Result<Member, Error> {
        self.request(
            Method::GET,
            format!("/servers/{server_id}/members/{user_id}"),
        )
        .response()
        .await
    }

    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<(), Error> {
        self.request(
            Method::DELETE,
            format!("/channels/{channel_id}/messages/{message_id}"),
        )
        .send()
        .await
    }

    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        data: &DataEditMessage,
    ) -> Result<Message, Error> {
        self.request(
            Method::PATCH,
            format!("/channels/{}/messages/{}", channel_id, message_id),
        )
        .body(&data)
        .response()
        .await
    }

    pub async fn join_call(
        &self,
        channel_id: &str,
        data: &DataJoinCall,
    ) -> Result<CreateVoiceUserResponse, Error> {
        self.request(Method::POST, format!("/channels/{channel_id}/join_call"))
            .body(data)
            .response()
            .await
    }

    pub async fn fetch_message(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<Message, Error> {
        self.request(
            Method::GET,
            format!("/channels/{channel_id}/messages/{message_id}"),
        )
        .response()
        .await
    }
}

pub struct HttpRequest {
    builder: RequestBuilder,
}

impl HttpRequest {
    pub fn body<I: Serialize>(mut self, body: &I) -> HttpRequest {
        self.builder = self.builder.json(body);

        self
    }

    pub fn query<I: Serialize>(mut self, query: &I) -> HttpRequest {
        self.builder = self.builder.query(query);

        self
    }

    pub async fn execute(self) -> Result<Response, Error> {
        let response = self.builder.send().await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let text = response.json().await?;
            Err(Error::HttpError(text))
        } else {
            Ok(response)
        }
    }

    pub async fn response<O: for<'a> Deserialize<'a>>(self) -> Result<O, Error> {
        self.execute().await?.json().await.map_err(Into::into)
    }

    pub async fn send(self) -> Result<(), Error> {
        self.execute().await?;

        Ok(())
    }
}
