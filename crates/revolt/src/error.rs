use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Error {
    HttpError(Arc<reqwest::Error>),
    MissingParameter,
    ConverterError(String),
    Timeout,
    BrokenChannel,
    InternalError
}
