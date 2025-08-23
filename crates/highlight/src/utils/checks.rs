use revolt::{commands::Context, types::Channel};

use crate::{Error, State};

pub async fn raise_if_not_in_server<'a>(ctx: &'a Context<Error, State>) -> Result<String, Error> {
    let cache = ctx.cache.read().await;

    let channel = cache.channels.get(&ctx.message.channel)
        .ok_or(Error::NotInServer)?;

    match channel {
        Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => Ok(server.clone()),
        _ => Err(Error::NotInServer)
    }
}