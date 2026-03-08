use std::{fmt::Display, sync::Arc};

use stoat_permissions::ChannelPermission;
pub use stoat_result::{Error as StoatHttpError, ErrorType as StoatHttpErrorType};

use crate::types::RatelimitFailure;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// All possible errors for Stoat-rs, this includes errors from dependancies like `reqwest`, `tungestenite` and `livekit`.
///
/// Your custom error type should implement `From<stoat::Error>` to allow errors to automatically travel up the call chain throughout your codebase.
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
    InvalidTag,
    MalformedID,
    InvalidUrl,

    NotAudioTrack,
    NotVideoTrack,

    Close,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ReqwestError(error) => write!(f, "Reqwest Error: {error}"),
            Error::HttpError(error) => write!(f, "HTTP Error: {error}"),
            Error::RatelimitReached(ratelimit_failure) => write!(f, "Ratelimit Reached: try again in {}ms", ratelimit_failure.retry_after),
            Error::WsError(error) => write!(f, "Websocket Error: {error}"),
            #[cfg(feature = "voice")]
            Error::LiveKit(error) => write!(f, "Livekit Room Error: {error}"),
            Error::MissingParameter => write!(f, "Missing Required Parameter"),
            Error::ConverterError(error) => write!(f, "Converter Error: {error}"),
            Error::Timeout => write!(f, "Timed out"),
            Error::BrokenChannel => write!(f, "Broken Channel"),
            Error::InternalError => write!(f, "Interal Error"),
            Error::CheckFailure => write!(f, "Check Failure"),
            Error::MissingChannelPermission { permissions } => write!(f, "Missing Required Permission {permissions}"),
            Error::NotInServer => write!(f, "Not In Server"),
            Error::NotInDM => write!(f, "Not In DMs"),
            Error::NotOwner => write!(f, "Not Owner"),
            Error::NotNsfw => write!(f, "Not In NSFW Channel"),
            Error::InvalidTag => write!(f, "Invalid File Tag"),
            Error::MalformedID => write!(f, "Malformed ULID ID"),
            Error::InvalidUrl => write!(f, "Invalid URL"),
            Error::NotAudioTrack => write!(f, "Not Audio Track"),
            Error::NotVideoTrack => write!(f, "Not Video Track"),
            Error::Close => write!(f, "Close"),
        }
    }
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

#[cfg(feature = "voice")]
impl From<livekit::webrtc::RtcError> for Error {
    fn from(value: livekit::webrtc::RtcError) -> Self {
        Self::LiveKit(Arc::new(livekit::RoomError::Rtc(value)))
    }
}
