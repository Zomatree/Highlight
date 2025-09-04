use revolt::{
    commands::{server_only, Command, Context, HasChannelPermissions},
    permissions::ChannelPermission,
    types::User,
};

use crate::{Error, State};

async fn view(ctx: Context<Error, State>, user: User) -> Result<(), Error> {
    let server_id = &ctx.get_current_server().await.as_ref().unwrap().id;

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
        .check(server_only)
        .check(HasChannelPermissions::new(vec![
            ChannelPermission::ManageMessages,
        ]))
}
