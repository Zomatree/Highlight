use revolt::{command, commands::Context};

use crate::{Error, State};

#[command(
    name = "help",
    error = Error,
    state = State,
    description = "Shows all commands"
)]
pub async fn help(ctx: &Context<Error, State>) -> Result<(), Error> {
    let commands = ctx
        .commands
        .read()
        .await
        .values()
        .map(|command| {
            format!(
                "- {} - {}",
                command.name.clone(),
                command.description.as_ref().unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    ctx.http
        .send_message(&ctx.message.channel)
        .content(format!("All commands:\n{commands}"))
        .build()
        .await?;

    Ok(())
}
