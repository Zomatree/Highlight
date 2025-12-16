use stoat::{
    ChannelExt, Client, Context, EventHandler, async_trait,
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
    ctx.get_current_channel()?
        .send(&ctx.http)
        .content("Pong!".to_string())
        .reply(ctx.message.id.clone(), true)
        .build()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let commands = CommandHandler::new(Commands, ())
        .with_static_prefix("!")
        .register(vec![Command::new("ping", ping)]);

    let events = Events(commands);

    let client = Client::new(events).await?;

    client.run("TOKEN HERE").await
}
