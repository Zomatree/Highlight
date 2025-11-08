use std::sync::Arc;

use stoat_permissions::ChannelPermission;

#[derive(Debug, Clone)]
pub enum Error {
    HttpError(Arc<reqwest::Error>),
    WsError(Arc<tungstenite::Error>),
    MissingParameter,
    ConverterError(String),
    Timeout,
    BrokenChannel,
    InternalError,
    CheckFailure,
    MissingChannelPermission { permissions: ChannelPermission },
    NotInServer,
    NotInDM,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::HttpError(Arc::new(value))
    }
}

impl From<tungstenite::Error> for Error {
    fn from(value: tungstenite::Error) -> Self {
        Self::WsError(Arc::new(value))
    }
}