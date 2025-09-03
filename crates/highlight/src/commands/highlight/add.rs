use revolt::commands::{Command, ConsumeRest, Context};

use crate::{Error, State, raise_if_not_in_server};

async fn add(ctx: Context<Error, State>, ConsumeRest(keyword): ConsumeRest) -> Result<(), Error> {
    let server_id = raise_if_not_in_server(&ctx).await?;

    match ctx.state
        .add_keyword(ctx.message.author.clone(), server_id, keyword)
        .await
    {
        Ok(_) => {
            ctx.http
                .send_message(&ctx.message.channel)
                .content("Added to your highlights.".to_string())
                .build()
                .await?;
        },
        Err(Error::PgError(e)) => {
            if e.as_database_error().unwrap().is_unique_violation() {
                ctx.http.send_message(&ctx.message.channel)
                    .content("Keyword already exists.".to_string())
                    .build()
                    .await?;
            }
        },
        res => return res
    }

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("add", add)
        .description("Adds a highlight keyword.")
        .signature("<keyword>")
}