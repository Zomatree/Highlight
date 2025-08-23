use revolt::{command, commands::Context, types::User};

use crate::{raise_if_not_in_server, Error, State};

#[command(
    name = "unblock",
    error = Error,
    state = State,
    description = "Blocks a user from highlighting you",
)]
pub async fn unblock(ctx: &mut Context<Error, State>, user: User) -> Result<(), Error> {
    ctx.http.send_message(&ctx.message.channel)
        .content(format!("{user:?}"))
        .build()
        .await?;

    Ok(())
}