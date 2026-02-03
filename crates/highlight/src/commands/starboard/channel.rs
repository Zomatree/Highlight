use stoat::{
    ChannelExt,
    commands::{Command, Context, server_only},
    types::Channel,
};

use crate::{Error, State};

async fn channel(ctx: Context<Error, State>, channel: Option<Channel>) -> Result<(), Error> {
    if let Some(channel) = channel {
        let server_id = ctx.get_current_server()?.id;

        if match &channel {
            Channel::TextChannel { server, .. } => server != &server_id,
            _ => true,
        } {
            ctx.get_current_channel()?
                .send(&ctx)
                .content("Invalid channel".to_string())
                .build()
                .await?;

            return Ok(());
        }

        ctx.state
            .update_server_config_starboard_channel(&server_id, channel.id())
            .await?;

        ctx.get_current_channel()?
            .send(&ctx)
            .content(format!("Starboard channel set to {}.", channel.mention()))
            .build()
            .await?;
    } else {
        let command = ctx.commands.get_command("starboard").unwrap();
        command.invoke(ctx.clone()).await?;
    };

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("channel", channel)
        .description("Sets the starboard channel.")
        .check(server_only)
}
