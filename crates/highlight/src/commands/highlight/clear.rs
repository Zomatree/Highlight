use std::time::Duration;

use stoat::{
    ChannelExt,
    commands::{Command, Context, server_only},
};

use crate::{Error, State, utils::MessageExt};

async fn clear(ctx: Context<Error, State>) -> Result<(), Error> {
    let server_id = ctx.get_current_server().unwrap().id;
    let user = ctx.get_user().await?;

    let keywords = ctx.state.clear_keywords(&user.id, &server_id).await?;

    ctx.get_current_channel()?
        .send(&ctx)
        .content(format!("Cleared {} keywords", keywords.len()))
        .build()
        .await?
        .delete_after(&ctx, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("clear", clear)
        .description("Clears all keywords in this server.")
        .check(server_only)
}
