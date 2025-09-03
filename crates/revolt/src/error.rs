use std::sync::Arc;

use revolt_permissions::ChannelPermission;

#[derive(Debug, Clone)]
pub enum Error {
    HttpError(Arc<reqwest::Error>),
    MissingParameter,
    ConverterError(String),
    Timeout,
    BrokenChannel,
    InternalError,
    CheckFailure,
    MissingChannelPermission { permissions: ChannelPermission },
}
