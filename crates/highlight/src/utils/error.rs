#[derive(Debug)]
pub enum Error {
    RevoltError(revolt::Error),
    PgError(sqlx::Error),
    NotInServer
}

impl From<revolt::Error> for Error {
    fn from(value: revolt::Error) -> Self {
        Self::RevoltError(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::PgError(value)
    }
}