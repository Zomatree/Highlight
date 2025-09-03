use revolt::commands::{Command, ConsumeRest, Context};

use crate::{Error, State, raise_if_not_in_server};

async fn remove(
    ctx: Context<Error, State>,
    ConsumeRest(keyword): ConsumeRest,
) -> Result<(), Error> {
    let server_id = raise_if_not_in_server(&ctx).await?;

    let removed = ctx
        .state
        .remove_keyword(ctx.message.author.clone(), server_id, keyword)
        .await?;

    if removed {
        ctx.http
            .send_message(&ctx.message.channel)
            .content("Removed from your highlights.".to_string())
            .build()
            .await?;
    } else {
        ctx.http
            .send_message(&ctx.message.channel)
            .content("Keyword doesnt exist.".to_string())
            .build()
            .await?;
    };

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("remove", remove)
        .description("Removes a highlighted keyword.")
        .signature("<keyword>")
        .signature("<user>")
}
