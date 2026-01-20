use std::time::Duration;

use stoat::{
    ChannelExt,
    commands::{Command, Context, server_only},
};

use crate::{Error, State, utils::MessageExt};

mod add;
mod block;
mod clear;
mod remove;
mod unblock;
mod view;

async fn highlight(ctx: Context<Error, State>) -> Result<(), Error> {
    let server_id = ctx.get_current_server()?.id;

    let highlights = ctx
        .state
        .fetch_keywords_for_user(&ctx.message.author, &server_id)
        .await?
        .into_iter()
        .map(|keyword| format!("- {keyword}"))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.get_current_channel()?
        .send(&ctx)
        .content(format!("Your highlights are:\n{highlights}"))
        .build()
        .await?
        .delete_after(&ctx, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("highlight", highlight)
        .alias("hl")
        .description("Manage highlight keywords.")
        .check(server_only)
        .child(add::command())
        .child(remove::command())
        .child(block::command())
        .child(unblock::command())
        .child(view::command())
        .child(clear::command())
}
