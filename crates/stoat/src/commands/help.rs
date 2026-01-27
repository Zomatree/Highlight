use std::fmt::Debug;

use async_trait::async_trait;
use stoat_models::v0::Message;

use crate::{
    Error,
    builders::SendMessageBuilder,
    commands::{Command, Context, Converter, command::CommandHandle},
};

#[async_trait]
pub trait HelpCommand<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>: Debug + Send + Sync
{
    async fn create_global_help(
        &self,
        context: Context<E, S>,
        commands: Vec<Command<E, S>>,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E>;
    async fn create_command_help(
        &self,
        context: Context<E, S>,
        command: Command<E, S>,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E>;
    async fn create_group_help(
        &self,
        context: Context<E, S>,
        command: Command<E, S>,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E>;

    async fn filter_commands(
        &self,
        context: Context<E, S>,
        commands: Vec<Command<E, S>>,
    ) -> Result<Vec<Command<E, S>>, E> {
        let mut filtered = Vec::new();

        for command in commands {
            if command.hidden {
                continue;
            };

            if command.can_run(context.clone()).await.is_ok_and(|b| b) {
                filtered.push(command);
            };
        }

        Ok(filtered)
    }

    #[allow(unused_variables)]
    async fn send_help_command(
        &self,
        context: Context<E, S>,
        builder: SendMessageBuilder,
    ) -> Result<Message, E> {
        Ok(builder.build().await?)
    }

    #[allow(unused_variables)]
    async fn after_help_command(&self, context: Context<E, S>, message: Message) -> Result<(), E> {
        Ok(())
    }

    async fn get_channel(&self, context: Context<E, S>) -> Result<String, E> {
        Ok(context.message.channel.clone())
    }

    async fn no_command_found(
        &self,
        context: Context<E, S>,
        name: String,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E>;
}

#[derive(Debug)]
pub struct DefaultHelpCommand;

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> HelpCommand<E, S> for DefaultHelpCommand
{
    async fn create_global_help(
        &self,
        _context: Context<E, S>,
        commands: Vec<Command<E, S>>,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E> {
        let mut lines = vec!["```".to_string()];

        for command in commands {
            lines.push(format!(
                "{} - {}",
                &command.name,
                command
                    .description
                    .as_ref()
                    .map(|desc| desc.split('\n').next().unwrap())
                    .unwrap_or("No description")
            ));
        }

        lines.push("```".to_string());

        builder.content(lines.join("\n"));

        Ok(())
    }

    async fn create_command_help(
        &self,
        context: Context<E, S>,
        command: Command<E, S>,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E> {
        let mut lines = vec!["```".to_string(), format!("{}:", &command.name)];

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

        lines.push("```".to_string());

        builder.content(lines.join("\n"));

        Ok(())
    }

    async fn create_group_help(
        &self,
        context: Context<E, S>,
        command: Command<E, S>,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E> {
        let mut lines = vec!["```".to_string(), format!("{}:", &command.name)];

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

        lines.push("```".to_string());

        builder.content(lines.join("\n"));

        Ok(())
    }

    async fn no_command_found(
        &self,
        _context: Context<E, S>,
        name: String,
        builder: &mut SendMessageBuilder,
    ) -> Result<(), E> {
        builder.content(format!("Command `{name}` not found."));

        Ok(())
    }
}

#[derive(Clone)]
struct HelpCommandImpl;

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> CommandHandle<(), E, S> for HelpCommandImpl
{
    async fn handle(&self, context: Context<E, S>) -> Result<(), E> {
        let args = Vec::<String>::from_context(&context).await?;

        let channel_id = context.help_command.get_channel(context.clone()).await?;
        let mut builder = SendMessageBuilder::new(context.http.clone(), channel_id);

        let commands = context
            .help_command
            .filter_commands(context.clone(), context.commands.get_commands())
            .await?;

        if args.is_empty() {
            context
                .help_command
                .create_global_help(context.clone(), commands, &mut builder)
                .await?;
        } else {
            if let Some(command) = context.commands.get_command_from_slice(&args) {
                if command.children.is_empty() {
                    context
                        .help_command
                        .create_command_help(context.clone(), command, &mut builder)
                        .await?;
                } else {
                    context
                        .help_command
                        .create_group_help(context.clone(), command, &mut builder)
                        .await?;
                }
            } else {
                context
                    .help_command
                    .no_command_found(context.clone(), args.join(" "), &mut builder)
                    .await?;
            }
        }

        let message = context
            .help_command
            .send_help_command(context.clone(), builder)
            .await?;
        context
            .help_command
            .after_help_command(context.clone(), message)
            .await?;

        Ok(())
    }
}

pub(crate) fn help_command<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>() -> Command<E, S> {
    Command::new("help", HelpCommandImpl)
        .signature("<command>")
        .description("Shows help for a command, group or all commands")
}
