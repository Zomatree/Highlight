use serde::{Deserialize, Serialize};

pub use stoat_models::v0::*;
pub use stoat_permissions::{ChannelPermission, PermissionValue, UserPermission, Override, OverrideField, DataPermissionsValue};

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
