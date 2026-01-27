use crate::{CmdCtx, Error};
use stoat::{
    MemberExt, MessageExt, ServerExt,
    commands::{Command, ConsumeRest, server_only},
    types::Member,
};

async fn ban(ctx: CmdCtx, member: Member, ConsumeRest(reason): ConsumeRest) -> Result<(), Error> {
    let reason = if reason.is_empty() {
        "No reason".to_string()
    } else {
        reason
    };

    member.ban(&ctx, Some(reason)).await?;

    ctx.message
        .reply(&ctx, false)
        .content(format!("Banned {}.", member.mention()))
        .build()
        .await?;

    Ok(())
}

async fn remove_ban(ctx: CmdCtx, user_id: String) -> Result<(), Error> {
    let server = ctx.get_current_server().unwrap();

    server.unban_member(&ctx, &user_id).await?;

    ctx.message
        .reply(&ctx, false)
        .content(format!("Unbanned <@{user_id}>."))
        .build()
        .await?;

    Ok(())
}

pub fn command() -> Command<Error, ()> {
    Command::new("ban", ban)
        .description("Bans a member.")
        .signature("<member> [reason...]")
        .check(server_only)
        .child(
            Command::new("remove", remove_ban)
                .description("Unbans a member.")
                .signature("<user_id>")
                .check(server_only),
        )
}
