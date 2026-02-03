use std::time::Duration;

use stoat::{
    ChannelExt, Identifiable,
    commands::{Greedy, HasChannelPermissions, server_only},
    either::Either,
    types::{ChannelPermission, OptionsBulkDelete, User},
    ulid::Ulid,
};

use crate::{CmdCtx, Command, Error, MessageExt, Result};

async fn purge(
    ctx: CmdCtx,
    Greedy(users): Greedy<Either<User, Ulid>>,
    limit: Option<u32>,
) -> Result<()> {
    if users.is_empty() && limit.is_none() {
        return Err(Error::UserError("No specified users or limit.".to_string()));
    };

    let limit = limit.unwrap_or(10).min(100);
    let channel = ctx.get_current_channel().unwrap();

    let messages = channel
        .fetch_messages(&ctx)
        .limit(limit as i64)
        .build()
        .await?
        .into_iter()
        .filter(|msg| users.is_empty() || users.iter().any(|user| user.id() == &msg.author))
        .map(|msg| msg.id)
        .collect::<Vec<_>>();

    let len = messages.len();

    if !messages.is_empty() {
        channel
            .delete_messages(&ctx, &OptionsBulkDelete { ids: messages })
            .await?;
    };

    ctx.send()
        .content(format!("Deleted {len} messages."))
        .build()
        .await?
        .delete_after(&ctx, Duration::from_secs(5));

    Ok(())
}

pub fn command() -> Command {
    Command::new("purge", purge)
        .description("Bulk deletes messages")
        .signature("<users...> <limit>")
        .hidden()
        .check(server_only)
        .check(HasChannelPermissions::new(vec![
            ChannelPermission::ManageMessages,
        ]))
}
