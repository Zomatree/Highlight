use stoat::{
    Client, Context, EventHandler, async_trait,
    commands::{CommandHandler, Context as CommandContext},
    types::Message,
};

mod commands;

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
struct Events(CommandHandler<commands::Commands>);

#[async_trait]
impl EventHandler for Events {
    type Error = Error;

    async fn message(&self, context: Context, message: Message) -> Result<(), Self::Error> {
        self.0.process_commands(context, message).await
    }
}

type CmdCtx = CommandContext<Error, ()>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let commands = CommandHandler::new(commands::Commands, ()).register(commands::commands());

    let events = Events(commands);

    Client::new(events).await?.run("TOKEN HERE").await
}
