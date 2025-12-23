use std::time::Duration;

use stoat::{
    ChannelExt,
    commands::{Command, ConsumeRest, Context, server_only},
};

use crate::{Error, State, utils::MessageExt};

async fn remove(
    ctx: Context<Error, State>,
    ConsumeRest(keyword): ConsumeRest,
) -> Result<(), Error> {
    let server_id = ctx.get_current_server()?.id;

    let removed = ctx
        .state
        .remove_keyword(ctx.message.author.clone(), server_id, keyword)
        .await?;

    if removed {
        ctx.get_current_channel()?
            .send(&ctx)
            .content("Removed from your highlights.".to_string())
            .build()
            .await?
    } else {
        ctx.get_current_channel()?
            .send(&ctx)
            .content("Keyword doesnt exist.".to_string())
            .build()
            .await?
    }
    .delete_after(&ctx, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("remove", remove)
        .description("Removes a highlighted keyword.")
        .signature("<keyword>")
        .check(server_only)
}
