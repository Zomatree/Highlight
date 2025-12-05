use std::time::Duration;

use stoat::{
    ChannelExt,
    commands::{Command, Context},
    types::User,
};

use crate::{Error, State, utils::MessageExt};

async fn unblock(ctx: Context<Error, State>, user: User) -> Result<(), Error> {
    ctx.state
        .unblock_user(ctx.message.author.clone(), user.id.clone())
        .await?;

    ctx.get_current_channel()
        .await?
        .send(&ctx.http)
        .content(format!("Unblocked {}", user.username))
        .build()
        .await?
        .delete_after(&ctx.http, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("unblock", unblock)
        .description("Unblocks a user from highlighting you.")
        .signature("<user>")
}
