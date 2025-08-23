use revolt::{command, commands::{ConsumeRest, Context}};

use crate::{raise_if_not_in_server, Error, State};


#[command(
    name = "add",
    error = Error,
    state = State,
    description = "Adds a highlight keyword"
)]
pub async fn add(ctx: &Context<Error, State>, keyword: ConsumeRest) -> Result<(), Error> {
    let server_id = raise_if_not_in_server(ctx).await?;

    ctx.state.add_keyword(ctx.message.author.clone(), server_id, keyword.0).await?;

    ctx.http.send_message(&ctx.message.channel)
        .content("Added to your highlights.".to_string())
        .build()
        .await?;

    Ok(())
}