use std::time::Duration;

use stoat::{
    ChannelExt,
    commands::{Command, Context, HasChannelPermissions, server_only},
    permissions::ChannelPermission,
    types::User,
};

use crate::{Error, State, utils::MessageExt};

async fn view(ctx: Context<Error, State>, user: User) -> Result<(), Error> {
    let server_id = ctx.get_current_server().await?.id;

    let highlights = ctx
        .state
        .fetch_keywords_for_user(&user.id, &server_id)
        .await?
        .into_iter()
        .map(|keyword| format!("- {keyword}"))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.get_current_channel()
        .await?
        .send(&ctx.http)
        .content(format!(
            "{}'s highlights are:\n{highlights}",
            &user.username
        ))
        .build()
        .await?
        .delete_after(&ctx.http, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("view", view)
        .description("Views the keywords a user has in this server.")
        .signature("<user>")
        .check(server_only)
        .check(HasChannelPermissions::new(vec![
            ChannelPermission::ManageMessages,
        ]))
}
