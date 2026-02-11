use std::borrow::Cow;

use stoat::{
    ChannelExt,
    commands::{Command, Context, HasServerPermissions, server_only},
    permissions::{calculate_channel_permissions, user_permissions_query},
    types::{Channel, ChannelPermission},
};

use crate::{Error, State};

async fn channel(ctx: Context<Error, State>, channel: Option<Channel>) -> Result<(), Error> {
    if let Some(channel) = channel {
        let server = ctx.get_current_server()?;

        if match &channel {
            Channel::TextChannel {
                server: server_id, ..
            } => server_id != &server.id,
            _ => true,
        } {
            ctx.send()
                .content("Invalid channel".to_string())
                .build()
                .await?;

            return Ok(());
        }

        let user = ctx.get_user().await?;
        let member = ctx.get_member().await?;

        let mut query =
            user_permissions_query(ctx.cache.clone(), ctx.http.clone(), Cow::Borrowed(&user))
                .channel(Cow::Borrowed(&channel))
                .server(Cow::Borrowed(&server))
                .member(Cow::Borrowed(&member));

        let permissions = calculate_channel_permissions(&mut query).await;

        if !permissions.has_channel_permission(ChannelPermission::ManageChannel) {
            ctx.send()
                .content("You do not `ManageChannel` permission in that channel.".to_string())
                .build()
                .await?;

            return Ok(());
        }

        ctx.state
            .update_server_config_starboard_channel(&server.id, channel.id())
            .await?;

        ctx.send()
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
        .check(HasServerPermissions::new(vec![
            ChannelPermission::ManageChannel,
        ]))
}
