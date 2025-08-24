use revolt::{command, commands::Context, types::User};

use crate::{Error, State};

#[command(
    name = "unblock",
    error = Error,
    state = State,
    description = "Blocks a user from highlighting you",
)]
pub async fn unblock(ctx: &mut Context<Error, State>, user: User) -> Result<(), Error> {
    ctx.state
        .unblock_user(ctx.message.author.clone(), user.id.clone())
        .await?;

    ctx.http
        .send_message(&ctx.message.channel)
        .content(format!("Unblocked {}", user.username))
        .build()
        .await?;

    Ok(())
}
