use serde::{Deserialize, Serialize};

pub use stoat_database::events::{
    client::{EventV1, Ping},
    server::ClientMessage,
};
pub use stoat_models::v0::*;
pub use stoat_permissions::{
    ChannelPermission, DataPermissionsValue, Override, OverrideField, PermissionValue,
    UserPermission,
};

use crate::Error;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CaptchaFeature {
    pub enabled: bool,
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Feature {
    pub enabled: bool,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoiceNode {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub public_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoiceFeature {
    pub enabled: bool,
    pub nodes: Vec<VoiceNode>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StoatFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Feature,
    pub january: Feature,
    pub livekit: VoiceFeature,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BuildInformation {
    pub commit_sha: String,
    pub commit_timestamp: String,
    pub semver: String,
    pub origin_url: String,
    pub timestamp: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StoatConfig {
    pub revolt: String,
    pub features: StoatFeatures,
    pub ws: String,
    pub app: String,
    pub vapid: String,
    pub build: BuildInformation,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AutumnResponse {
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RatelimitFailure {
    pub retry_after: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum Tag {
    Attachments,
    Avatars,
    Backgrounds,
    Icons,
    Banners,
    Emojis,
}

impl Tag {
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    pub fn to_string(self) -> String {
        self.as_str().to_string()
    }
}

impl Into<&'static str> for Tag {
    fn into(self) -> &'static str {
        match self {
            Tag::Attachments => "attachments",
            Tag::Avatars => "avatars",
            Tag::Backgrounds => "backgrounds",
            Tag::Icons => "icons",
            Tag::Banners => "banners",
            Tag::Emojis => "emojis",
        }
    }
}

impl TryFrom<&str> for Tag {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "attachments" => Ok(Tag::Attachments),
            "avatars" => Ok(Tag::Avatars),
            "backgrounds" => Ok(Tag::Backgrounds),
            "icons" => Ok(Tag::Icons),
            "banners" => Ok(Tag::Banners),
            "emojis" => Ok(Tag::Emojis),
            _ => Err(Error::InvalidTag),
        }
    }
}
