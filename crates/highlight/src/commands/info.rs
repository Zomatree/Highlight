use stoat::{
    ChannelExt,
    commands::{Command, Context},
};

use crate::{Error, State};

async fn info(ctx: Context<Error, State>) -> Result<(), Error> {
    let server_count = ctx.cache.servers.len();
    let trigger_word_count = ctx.state.get_total_keyword_count().await?;

    ctx.get_current_channel()?
        .send(&ctx)
        .content(format!(
            "\
# Highlight
Lets users create trigger words and be alerted when those triggers are said.

Running in `{server_count}` servers!
There are `{trigger_word_count}` trigger words in my database.

*My source code can be found at [Zomatree/Highlight](<https://github.com/Zomatree/Highlight>).*"
        ))
        .build()
        .await?;

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("info", info).description("Misc info about bot")
}
