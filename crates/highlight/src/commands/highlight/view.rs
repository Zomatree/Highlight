use revolt::{
    commands::{Command, Context, HasChannelPermissions},
    permissions::ChannelPermission,
    types::User,
};

use crate::{Error, State, raise_if_not_in_server};

async fn view(ctx: Context<Error, State>, user: User) -> Result<(), Error> {
    let server_id = raise_if_not_in_server(&ctx).await?;

    let highlights = ctx
        .state
        .fetch_keywords_for_user(&user.id, &server_id)
        .await?
        .into_iter()
        .map(|keyword| format!("- {keyword}"))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.http
        .send_message(&ctx.message.channel)
        .content(format!(
            "{}'s highlights are:\n{highlights}",
            &user.username
        ))
        .build()
        .await?;
    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("view", view)
        .description("Views the keywords a user has in this server.")
        .signature("<user>")
        .check(HasChannelPermissions::new(vec![
            ChannelPermission::ManageMessages,
        ]))
}
