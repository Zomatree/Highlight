use stoat::{
    ChannelExt,
    commands::{Command, Context, server_only},
};

use crate::{Error, State};

async fn channel(ctx: Context<Error, State>, limit: Option<u32>) -> Result<(), Error> {
    if let Some(limit) = limit {
        if limit == 0 {
            ctx.get_current_channel()?
                .send(&ctx)
                .content("Invalid limit".to_string())
                .build()
                .await?;

            return Ok(());
        }

        let server_id = ctx.get_current_server()?.id;
        ctx.state
            .update_server_config_star_count(&server_id, limit as i32)
            .await?;

        ctx.get_current_channel()?
            .send(&ctx)
            .content(format!("Starboard star limit set to {limit}."))
            .build()
            .await?;
    } else {
        let command = ctx.commands.get_command("starboard").unwrap();
        command.invoke(ctx.clone()).await?;
    };

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("limit", channel)
        .description("Sets the minimum required stars for a message.")
        .check(server_only)
}
