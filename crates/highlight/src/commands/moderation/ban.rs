use stoat::{
    Identifiable, Ulid,
    commands::{ConsumeRest, HasServerPermissions, server_only},
    either::Either,
    types::{ChannelPermission, DataBanCreate, Member},
};

use crate::{CmdCtx, Command, Result};

pub async fn ban(
    ctx: CmdCtx,
    member: Either<Member, Ulid>,
    ConsumeRest(reason): ConsumeRest,
) -> Result<()> {
    let server = ctx.get_current_server()?;

    ctx.http
        .ban_member(
            &server.id,
            member.id(),
            &DataBanCreate {
                reason: Some(if reason.is_empty() {
                    "No reason".to_string()
                } else {
                    reason
                }),
            },
        )
        .await?;

    ctx.send()
        .content(format!("Banned <@{}>.", member.id(),))
        .build()
        .await?;

    Ok(())
}

pub fn command() -> Command {
    Command::new("ban", ban)
        .description("Bans a member.")
        .signature("<member> <reason...>")
        .hidden()
        .check(server_only)
        .check(HasServerPermissions::new(vec![
            ChannelPermission::BanMembers,
        ]))
}
