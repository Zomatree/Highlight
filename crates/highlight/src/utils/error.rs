use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Error {
    StoatError(stoat::Error),
    PgError(Arc<sqlx::Error>),
    InvalidKeyword
}

impl From<stoat::Error> for Error {
    fn from(value: stoat::Error) -> Self {
        Self::StoatError(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::PgError(Arc::new(value))
    }
}
