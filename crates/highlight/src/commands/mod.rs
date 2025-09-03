use revolt::{
    async_trait, commands::{Command, CommandEventHandler, Context},
};

use crate::{Error, State};

mod help;
mod highlight;

#[derive(Clone)]
pub struct CommandEvents;

#[async_trait]
impl CommandEventHandler<Error, State> for CommandEvents {
    async fn error(&self, ctx: Context<Error, State>, error: Error) -> Result<(), Error> {
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

async fn test(ctx: Context<Error, State>) -> Result<(), Error> {
    let msg = ctx
        .notifiers
        .wait_for_message({
            let author = ctx.message.author.clone();
            let channel = ctx.message.channel.clone();

            move |msg| msg.author == author && msg.channel == channel
        },
        None)
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
        Command::new("test", test)
            .description("Test command"),

        help::command(),
        highlight::command()
    ]
}
