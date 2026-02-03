use crate::{CmdCtx, Error, Result, State};
use std::time::Duration;
use stoat::{async_trait, commands::Converter};

pub struct DurationConverter(pub Duration);

#[async_trait]
impl Converter<Error, State> for DurationConverter {
    async fn convert(_context: &CmdCtx, input: String) -> Result<Self> {
        humantime::parse_duration(&input)
            .map(Self)
            .map_err(|e| Error::UserError(e.to_string()))
    }
}
