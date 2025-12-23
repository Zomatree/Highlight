use std::time::Duration;

use stoat::{
    ChannelExt,
    commands::{Command, Context},
    types::User,
};

use crate::{Error, State, utils::MessageExt};

async fn block(ctx: Context<Error, State>, user: User) -> Result<(), Error> {
    ctx.state
        .block_user(ctx.message.author.clone(), user.id.clone())
        .await?;

    ctx.get_current_channel()?
        .send(&ctx)
        .content(format!("Blocked {}", user.username))
        .build()
        .await?
        .delete_after(&ctx, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("block", block)
        .description("Blocks a user from highlighting you.")
        .signature("<user>")
}
