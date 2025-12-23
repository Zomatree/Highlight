use std::time::Duration;

use stoat::{
    ChannelExt, async_trait,
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
        let Some(command) = ctx.command.as_ref() else {
            return Ok(());
        };

        if command.parents.get(0).is_some_and(|p| p == "highlight") {
            ctx.message.delete_after(&ctx, Duration::from_secs(5));
        };

        Ok(())
    }

    async fn error(&self, ctx: Context<Error, State>, error: Error) -> Result<(), Error> {
        match error {
            Error::StoatError(stoat::Error::NotInServer) => {
                ctx.get_current_channel()?
                    .send(&ctx)
                    .content("This command can only be used in a server".to_string())
                    .build()
                    .await?;
            }
            error => log::error!("{error:?}"),
        };

        Ok(())
    }
}


pub fn commands() -> Vec<Command<Error, State>> {
    vec![
        help::command(),
        highlight::command(),
        info::command(),
    ]
}
