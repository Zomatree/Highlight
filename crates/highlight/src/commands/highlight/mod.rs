use revolt::commands::{server_only, Command, Context};

use crate::{Error, State};

mod add;
mod block;
mod remove;
mod unblock;
mod view;

async fn highlight(ctx: Context<Error, State>) -> Result<(), Error> {
    let server_id =  &ctx.get_current_server().await.as_ref().unwrap().id;

    let highlights = ctx
        .state
        .fetch_keywords_for_user(&ctx.message.author, server_id)
        .await?
        .into_iter()
        .map(|keyword| format!("- {keyword}"))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.http
        .send_message(&ctx.message.channel)
        .content(format!("Your highlights are:\n{highlights}"))
        .build()
        .await?;

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
}
