use stoat::{
    Client, Context, EventHandler, MessageExt, async_trait,
    commands::{Command, CommandEventHandler, CommandHandler, Context as CommandContext},
    types::Message,
};

#[derive(Debug, Clone)]
pub enum Error {
    StoatError(stoat::Error),
}

impl From<stoat::Error> for Error {
    fn from(value: stoat::Error) -> Self {
        Self::StoatError(value)
    }
}

#[derive(Clone)]
struct Commands;

#[async_trait]
impl CommandEventHandler for Commands {
    type State = ();
    type Error = Error;

    async fn get_prefix(&self, _ctx: CmdCtx) -> Result<Vec<String>, Error> {
        Ok(vec!["!".to_string()])
    }
}

#[derive(Clone)]
struct Events(CommandHandler<Commands>);

#[async_trait]
impl EventHandler for Events {
    type Error = Error;

    async fn ready(&self, context: Context) -> Result<(), Self::Error> {
        println!(
            "Logged into {}",
            context.cache.get_current_user().unwrap().username
        );

        Ok(())
    }

    async fn message(&self, context: Context, message: Message) -> Result<(), Self::Error> {
        self.0.process_commands(context, message).await
    }
}

type CmdCtx = CommandContext<Error, ()>;

async fn ping(ctx: CmdCtx) -> Result<(), Error> {
    ctx.message
        .reply(&ctx, true)
        .content("Pong!".to_string())
        .build()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let commands = CommandHandler::new(Commands, ()).register(vec![Command::new("ping", ping)]);

    let events = Events(commands);

    Client::new(events).await?.run("TOKEN HERE").await
}
