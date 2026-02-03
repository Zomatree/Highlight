use iso8601_timestamp::Timestamp;
use stoat::{
    MemberExt,
    commands::{ConsumeRest, HasServerPermissions, server_only},
    types::{ChannelPermission, Member},
};

use crate::{CmdCtx, Command, DurationConverter, Result};

pub async fn timeout(
    ctx: CmdCtx,
    member: Member,
    ConsumeRest(DurationConverter(duration)): ConsumeRest<DurationConverter>,
) -> Result<()> {
    let Ok(iso_duration) = duration.try_into() else {
        return Err(crate::Error::UserError("Duration too long".to_string()));
    };

    let Some(timestamp) = Timestamp::now_utc().checked_add(iso_duration) else {
        return Err(crate::Error::UserError("Duration too long".to_string()));
    };

    member.edit(&ctx).timeout(Some(timestamp)).build().await?;

    ctx.send()
        .content(format!(
            "Timed out {} for {}.",
            member.mention(),
            humantime::format_duration(duration)
        ))
        .build()
        .await?;

    Ok(())
}

pub async fn remove(ctx: CmdCtx, member: Member) -> Result<()> {
    if member.timeout.is_none() {
        ctx.send()
            .content("Member is not timed out.".to_string())
            .build()
            .await?;

        return Ok(());
    }

    member.edit(&ctx).timeout(None).build().await?;

    ctx.send()
        .content(format!("Removed timeout for {}", member.mention()))
        .build()
        .await?;

    Ok(())
}

pub fn command() -> Command {
    Command::new("timeout", timeout)
        .description("Times out a member for a period of time.")
        .signature("<member> <duration...>")
        .hidden()
        .child(
            Command::new("remove", remove)
                .description("Removes a timeout from a member.")
                .signature("<member>")
                .check(server_only)
                .check(HasServerPermissions::new(vec![
                    ChannelPermission::TimeoutMembers,
                ])),
        )
        .check(server_only)
        .check(HasServerPermissions::new(vec![
            ChannelPermission::TimeoutMembers,
        ]))
}
