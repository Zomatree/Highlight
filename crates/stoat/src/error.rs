use std::sync::Arc;

use serde::Deserialize;
use stoat_permissions::ChannelPermission;

#[derive(Deserialize, Debug, Clone)]
pub struct HttpError {
    pub r#type: String,
    pub location: String,
}

#[derive(Debug, Clone)]
pub enum Error {
    ReqwestError(Arc<reqwest::Error>),
    HttpError(HttpError),
    WsError(Arc<tungstenite::Error>),
    #[cfg(feature = "voice")]
    LiveKit(Arc<livekit::RoomError>),
    MissingParameter,
    ConverterError(String),
    Timeout,
    BrokenChannel,
    InternalError,
    CheckFailure,
    MissingChannelPermission {
        permissions: ChannelPermission,
    },
    NotInServer,
    NotInDM,

    NotAudioTrack,
    NotVideoTrack,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(Arc::new(value))
    }
}

impl From<tungstenite::Error> for Error {
    fn from(value: tungstenite::Error) -> Self {
        Self::WsError(Arc::new(value))
    }
}

#[cfg(feature = "voice")]
impl From<livekit::RoomError> for Error {
    fn from(value: livekit::RoomError) -> Self {
        Self::LiveKit(Arc::new(value))
    }
}
