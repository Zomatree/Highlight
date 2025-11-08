use std::time::Duration;

use stoat::{
    async_trait,
    commands::{Command, CommandEventHandler, Context},
};

use crate::{Error, State, utils::MessageExt};

mod help;
mod highlight;
mod info;

#[derive(Clone)]
pub struct CommandEvents;

#[async_trait]
impl CommandEventHandler for CommandEvents {
    type Error = Error;
    type State = State;

    async fn after_command(&self, ctx: Context<Error, State>) -> Result<(), Error> {
        let Some(command) = ctx.command.as_ref() else { return Ok(()) };

        if command.parents.get(0).is_some_and(|p| p == "highlight") {
            ctx.message.delete_after(&ctx.http, Duration::from_secs(5));
        };

        Ok(())
    }

    async fn error(&self, ctx: Context<Error, State>, error: Error) -> Result<(), Error> {
        match error {
            Error::StoatError(stoat::Error::NotInServer) => {
                ctx.http
                    .send_message(&ctx.message.channel)
                    .content("This command can only be used in a server".to_string())
                    .build()
                    .await?;
            }
            error => log::error!("{error:?}"),
        };

        Ok(())
    }
}

async fn test(ctx: Context<Error, State>) -> Result<(), Error> {
    let msg = ctx
        .notifiers
        .wait_for_message(
            {
                let author = ctx.message.author.clone();
                let channel = ctx.message.channel.clone();

                move |msg| msg.author == author && msg.channel == channel
            },
            None,
        )
        .await?;

    ctx.http
        .send_message(&ctx.message.channel)
        .content(msg.content.unwrap())
        .build()
        .await?;

    Ok(())
}

pub fn commands() -> Vec<Command<Error, State>> {
    vec![
        Command::new("test", test).description("Test command."),
        help::command(),
        highlight::command(),
        info::command(),
    ]
}
