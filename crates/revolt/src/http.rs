use futures::TryFutureExt;
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::{builders::send_message::SendMessageBuilder, error::Error};

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
pub struct VoiceFeature {
    pub enabled: bool,
    pub url: String,
    pub ws: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RevoltFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Feature,
    pub january: Feature,
    pub voso: VoiceFeature,
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
pub struct RevoltConfig {
    pub revolt: String,
    pub features: RevoltFeatures,
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
            .request(method, format!("{}{}", &self.base, route.as_ref()));

        if let Some(token) = &self.token {
            builder = builder.header("x-bot-token", token);
        }

        HttpRequest { builder }
    }

    pub async fn get_root(&self) -> Result<RevoltConfig, Error> {
        self.request(Method::GET, "/").response().await
    }

    pub fn send_message<'a>(&'a self, channel_id: &'a str) -> SendMessageBuilder<'a> {
        SendMessageBuilder::new(self, channel_id)
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

    pub async fn response<O: for<'a> Deserialize<'a>>(self) -> Result<O, Error> {
        self.builder
            .send()
            .and_then(|body| body.json())
            .map_err(Error::HttpError)
            .await
    }

    pub async fn send(self) -> Result<(), Error> {
        self.builder.send().await.map_err(Error::HttpError)?;

        Ok(())
    }
}
