use std::sync::Arc;

use stoat_permissions::ChannelPermission;
pub use stoat_result::Error as StoatHttpError;

use crate::types::RatelimitFailure;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub enum Error {
    ReqwestError(Arc<reqwest::Error>),
    HttpError(StoatHttpError),
    RatelimitReached(RatelimitFailure),
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
    NotOwner,
    NotNsfw,

    NotAudioTrack,
    NotVideoTrack,

    Close,
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
