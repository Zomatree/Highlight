use revolt::commands::{Command, Context};

use crate::{Error, State, raise_if_not_in_server};

mod add;
mod remove;
mod block;
mod unblock;
mod view;

async fn highlight(ctx: Context<Error, State>) -> Result<(), Error> {
    let server_id = raise_if_not_in_server(&ctx).await?;

    let highlights = ctx
        .state
        .fetch_keywords_for_user(&ctx.message.author, &server_id)
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
        .description("Manage highlight keywords")
        .child(add::command())
        .child(remove::command())
        .child(block::command())
        .child(unblock::command())
        .child(view::command())
}