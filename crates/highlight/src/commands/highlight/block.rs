use revolt::{command, commands::Context, types::User};

use crate::{Error, State};

#[command(
    name = "block",
    error = Error,
    state = State,
    description = "Blocks a user from highlighting you",
)]
pub async fn block(ctx: &mut Context<Error, State>, user: User) -> Result<(), Error> {
    ctx.http.send_message(&ctx.message.channel)
        .content(format!("{user:?}"))
        .build()
        .await?;

    Ok(())
}