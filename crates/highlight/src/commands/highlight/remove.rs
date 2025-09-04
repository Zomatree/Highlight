use revolt::commands::{server_only, Command, ConsumeRest, Context};

use crate::{Error, State};

async fn remove(
    ctx: Context<Error, State>,
    ConsumeRest(keyword): ConsumeRest,
) -> Result<(), Error> {
    let server_id = ctx.get_current_server().await.as_ref().unwrap().id.clone();

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
        .check(server_only)
}
