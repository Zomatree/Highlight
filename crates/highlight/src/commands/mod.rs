use revolt::{async_trait, commands::{Context, Command, CommandEventHandler}, commands};

use crate::{Error, State};

mod highlight;

pub struct CommandEvents;

#[async_trait]
impl CommandEventHandler<Error, State> for CommandEvents {
    async fn error(&self, ctx: &mut Context<'_, Error, State>, error: Error) -> Result<(), Error> {
        match error {
            Error::NotInServer => {
                ctx.http.send_message(&ctx.message.channel)
                    .content("This command can only be used in a server".to_string())
                    .build()
                    .await?;
            },
            error => println!("{error:?}")
        };

        Ok(())
    }
}

pub fn commands() -> Vec<Command<Error, State>> {
    commands![
        highlight::highlight,
    ]
}