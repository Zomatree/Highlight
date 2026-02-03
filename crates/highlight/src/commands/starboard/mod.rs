use stoat::{
    ChannelExt,
    commands::{Command, Context, server_only},
};

use crate::{Error, State};

mod channel;
mod limit;

async fn starboard(ctx: Context<Error, State>) -> Result<(), Error> {
    let server_id = ctx.get_current_server()?.id;

    let config = ctx.state.fetch_server_config(&server_id).await?;

    ctx.get_current_channel()?
        .send(&ctx)
        .content(if let Some(channel) = &config.starboard_channel {
            format!(
                "Starboard channel is set to <#{channel}>, with a minimum star count of {}.",
                config.star_count
            )
        } else {
            "No starboard channel configured.".to_string()
        })
        .build()
        .await?;

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("starboard", starboard)
        .alias("sb")
        .description("Manage starboard channel.")
        .check(server_only)
        .child(channel::command())
        .child(limit::command())
}
