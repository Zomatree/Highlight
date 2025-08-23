use revolt::{command, commands::Context};

use crate::{raise_if_not_in_server, Error, State};

mod add;
mod block;
mod unblock;

#[command(
    name = "highlight",
    error = Error,
    state = State,
    children = [
        add::add,
        block::block,
        unblock::unblock,
    ],
    description = "Managed highlight keywords",
)]
pub async fn highlight(ctx: &mut Context<Error, State>) -> Result<(), Error> {
    let server_id = raise_if_not_in_server(ctx).await?;

    let highlights = ctx.state.fetch_keywords_for_user(&ctx.message.author, &server_id)
        .await?
        .into_iter()
        .map(|keyword| format!("- {keyword}"))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.http.send_message(&ctx.message.channel)
        .content(format!("Your highlights are:\n{highlights}"))
        .build()
        .await?;

    Ok(())
}