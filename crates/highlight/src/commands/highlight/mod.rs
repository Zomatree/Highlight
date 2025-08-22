use revolt::{command, commands::Context};

use crate::{raise_if_not_in_server, Error, State};

mod add;

#[command("highlight", Error, State, [add::add])]
pub async fn highlight(ctx: &mut Context<'_, Error, State>) -> Result<(), Error> {
    let server_id = raise_if_not_in_server(ctx)?;

    let highlights = ctx.state.get_keywords(ctx.message.author.clone(), server_id.to_string())
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