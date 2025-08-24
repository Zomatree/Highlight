use revolt::{
    async_trait, command, commands,
    commands::{Command, CommandEventHandler, Context},
};

use crate::{Error, State};

mod help;
mod highlight;

#[derive(Clone)]
pub struct CommandEvents;

#[async_trait]
impl CommandEventHandler<Error, State> for CommandEvents {
    async fn error(&self, ctx: &mut Context<Error, State>, error: Error) -> Result<(), Error> {
        match error {
            Error::NotInServer => {
                ctx.http
                    .send_message(&ctx.message.channel)
                    .content("This command can only be used in a server".to_string())
                    .build()
                    .await?;
            }
            error => println!("{error:?}"),
        };

        Ok(())
    }
}

#[command(name = "test", error = Error, state = State)]
async fn test(ctx: &mut Context<Error, State>) -> Result<(), Error> {
    let author_id = ctx.message.author.clone();

    let msg = ctx
        .waiters
        .wait_for_message(move |msg| msg.author == author_id, None)
        .await?;

    ctx.http
        .send_message(&ctx.message.channel)
        .content(msg.content.unwrap())
        // .content("Hello world".to_string())
        .build()
        .await?;

    Ok(())
}

pub fn commands() -> Vec<Command<Error, State>> {
    commands![highlight::highlight, help::help, test]
}
