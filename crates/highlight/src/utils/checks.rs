use revolt::{commands::Context, types::Channel};

use crate::{Error, State};

pub fn raise_if_not_in_server<'a>(ctx: &'a Context<'a, Error, State>) -> Result<&'a str, Error> {
    let channel = ctx.cache.channels.get(&ctx.message.channel)
        .ok_or(Error::NotInServer)?;

    match channel {
        Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => Ok(server),
        _ => Err(Error::NotInServer)
    }
}