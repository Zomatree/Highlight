use revolt::{command, commands::Context, types::User};

use crate::{Error, State};

#[command(
    name = "block",
    error = Error,
    state = State,
    description = "Blocks a user from highlighting you",
)]
pub async fn block(ctx: &mut Context<Error, State>, user: User) -> Result<(), Error> {
    ctx.state
        .block_user(ctx.message.author.clone(), user.id.clone())
        .await?;

    ctx.http
        .send_message(&ctx.message.channel)
        .content(format!("Blocked {}", user.username))
        .build()
        .await?;

    Ok(())
}
