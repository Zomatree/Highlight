use std::time::Duration;

use stoat::{
    ChannelExt,
    commands::{Command, ConsumeRest, Context, server_only},
};

use crate::{Error, MessageExt, State};

async fn add(ctx: Context<Error, State>, ConsumeRest(keyword): ConsumeRest) -> Result<(), Error> {
    let server_id = ctx.get_current_server().await.as_ref().unwrap().id.clone();

    let current_keywords = ctx
        .state
        .fetch_keywords_for_user(&ctx.message.author, &server_id)
        .await?;

    if current_keywords.len() >= ctx.state.config.limits.max_keywords {
        ctx.get_current_channel()
            .await?
            .send(&ctx.http)
            .content(format!(
                "Max keyword amount reached ({})",
                ctx.state.config.limits.max_keywords
            ))
            .build()
            .await?
            .delete_after(&ctx.http, Duration::from_secs(5));

        return Ok(());
    };

    match ctx
        .state
        .add_keyword(ctx.message.author.clone(), server_id, keyword)
        .await
    {
        Ok(_) => {
            ctx.get_current_channel()
                .await?
                .send(&ctx.http)
                .content("Added to your highlights.".to_string())
                .build()
                .await?
                .delete_after(&ctx.http, Duration::from_secs(5));
        }
        Err(Error::PgError(e)) if e.as_database_error().unwrap().is_unique_violation() => {
            ctx.get_current_channel()
                .await?
                .send(&ctx.http)
                .content("Keyword already exists.".to_string())
                .build()
                .await?
                .delete_after(&ctx.http, Duration::from_secs(5));
        }
        res => return res,
    };

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("add", add)
        .description("Adds a highlight keyword.")
        .signature("<keyword>")
        .check(server_only)
}
