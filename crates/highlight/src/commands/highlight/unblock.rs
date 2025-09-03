use revolt::{commands::{Command, Context}, types::User};

use crate::{Error, State};

async fn unblock(ctx: Context<Error, State>, user: User) -> Result<(), Error> {
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

pub fn command() -> Command<Error, State> {
    Command::new("unblock", unblock)
        .description("Unblocks a user from highlighting you.")
        .signature("<user>")
}