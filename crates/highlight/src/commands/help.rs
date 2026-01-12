use stoat::{
    ChannelExt,
    commands::{Command, Context},
};

use crate::{Error, State};

async fn help(ctx: Context<Error, State>, args: Vec<String>) -> Result<(), Error> {
    if args.is_empty() {
        let commands = ctx
            .commands
            .get_commands()
            .await
            .into_iter()
            .map(|command| {
                format!(
                    "- {} - {}",
                    command.name.clone(),
                    command.description.as_ref().unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        ctx.get_current_channel()?
            .send(&ctx)
            .content(format!("All commands:\n{commands}"))
            .build()
            .await?;
    } else {
        let Some(command) = ctx.commands.get_command_from_slice(&args).await else {
            ctx.get_current_channel()?
                .send(&ctx)
                .content(format!("Command not found!"))
                .build()
                .await?;

            return Ok(());
        };

        let aliases = if !command.aliases.is_empty() {
            format!(
                "\n\n*Aliases: {}*",
                command
                    .aliases
                    .iter()
                    .map(|alias| format!("`{alias}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        if command.children.is_empty() {
            ctx.get_current_channel()?
                .send(&ctx)
                .content(format!(
                    "## {} {}\n{}{}",
                    command.name,
                    command.signature.as_deref().unwrap_or_default(),
                    command.description.as_ref().unwrap(),
                    aliases,
                ))
                .build()
                .await?;
        } else {
            let subcommands = command
                .children
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

            ctx.get_current_channel()?
                .send(&ctx)
                .content(format!(
                    "## {} {}\n{}{}\n\n### Subcommands:\n{}",
                    command.name,
                    command.signature.as_deref().unwrap_or_default(),
                    command.description.as_ref().unwrap(),
                    aliases,
                    subcommands
                ))
                .build()
                .await?;
        }
    }

    Ok(())
}

pub fn command() -> Command<Error, State> {
    Command::new("help", help)
        .alias("h")
        .description("Shows this command.")
        .signature("[args]")
}
