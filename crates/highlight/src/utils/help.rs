use std::fmt::Debug;
use stoat::{async_trait, builders::SendMessageBuilder, commands::HelpCommand};

use crate::{CmdCtx, Command, Error, State};

#[derive(Debug)]
pub struct HighlightHelpCommand;

#[async_trait]
impl HelpCommand<Error, State> for HighlightHelpCommand {
    async fn create_global_help(
        &self,
        _context: CmdCtx,
        commands: Vec<Command>,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), Error> {
        let mut lines = vec!["### All commands:".to_string()];

        for command in commands {
            lines.push(format!(
                "- {} - {}",
                &command.name,
                command
                    .description
                    .as_ref()
                    .map(|desc| desc.split('\n').next().unwrap())
                    .unwrap_or("No description")
            ));
        }

        builder.content(lines.join("\n"));

        Ok(())
    }

    async fn create_command_help(
        &self,
        context: CmdCtx,
        command: Command,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), Error> {
        let mut lines = vec![format!("### {}:", &command.name)];

        let mut usage = command.parents.clone();
        usage.push(command.name.clone());
        usage.push(command.signature.clone().unwrap_or_default());

        lines.push(format!(
            "    Usage: {}{}",
            context.clean_prefix(),
            usage.join(" ")
        ));

        if !command.aliases.is_empty() {
            lines.push(format!("    Aliases: {}", command.aliases.join(", ")));
        }

        if let Some(description) = command.description.clone() {
            lines.push("".to_string());
            lines.push(description);
        }

        builder.content(lines.join("\n"));

        Ok(())
    }

    async fn create_group_help(
        &self,
        context: CmdCtx,
        command: Command,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), Error> {
        let mut lines = vec![format!("### {}:", &command.name)];

        let mut usage = command.parents.clone();
        usage.push(command.name.clone());
        usage.push(command.signature.clone().unwrap_or_default());

        lines.push(format!(
            "    Usage: {}{}",
            context.clean_prefix(),
            usage.join(" ")
        ));

        if !command.aliases.is_empty() {
            lines.push(format!("    Aliases: {}", command.aliases.join(", ")));
        }

        if let Some(description) = command.description.clone() {
            lines.push("".to_string());
            lines.push(description);
            lines.push("".to_string());
        }

        let children = self
            .filter_commands(context.clone(), command.children())
            .await?;

        if !children.is_empty() {
            lines.push("Commands:".to_string());
        };

        for command in children {
            lines.push(format!(
                "    {} - {}",
                &command.name,
                command
                    .description
                    .as_ref()
                    .map(|desc| desc.split('\n').next().unwrap())
                    .unwrap_or("No description")
            ));
        }

        builder.content(lines.join("\n"));

        Ok(())
    }

    async fn no_command_found(
        &self,
        _context: CmdCtx,
        name: String,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), Error> {
        builder.content(format!("Command `{name}` not found."));

        Ok(())
    }
}
