use revolt::commands::{Command, Context, Rest};

use crate::{Error, State};

async fn help(ctx: Context<Error, State>, Rest(args): Rest) -> Result<(), Error> {
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

        ctx.http
            .send_message(&ctx.message.channel)
            .content(format!("All commands:\n{commands}"))
            .build()
            .await?;
    } else {
        let Some(command) = ctx.commands.get_command_from_slice(&args).await else {
            ctx.http
                .send_message(&ctx.message.channel)
                .content(format!("Command not found!"))
                .build()
                .await?;

            return Ok(());
        };

        let aliases = if !command.aliases.is_empty() {
            format!("\n\n*Aliases: {}*", command.aliases.join(", "))
        } else {
            String::new()
        };

        if command.children.is_empty() {
            ctx.http
                .send_message(&ctx.message.channel)
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

            ctx.http
                .send_message(&ctx.message.channel)
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
