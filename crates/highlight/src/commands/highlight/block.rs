use std::time::Duration;

use stoat::{
    commands::{Command, Context},
    types::User,
};

use crate::{Error, State, utils::MessageExt};

async fn block(ctx: Context<Error, State>, user: User) -> Result<(), Error> {
    ctx.state
        .block_user(ctx.message.author.clone(), user.id.clone())
        .await?;

    ctx.http
        .send_message(&ctx.message.channel)
        .content(format!("Blocked {}", user.username))
        .build()
        .await?
        .delete_after(&ctx.http, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("block", block)
        .description("Blocks a user from highlighting you.")
        .signature("<user>")
}
