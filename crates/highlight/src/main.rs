use revolt::{
    Client, Context as MessageContext, Error as RevoltError, EventHandler, async_trait, command,
    commands,
    commands::{CommandEventHandler, CommandHandler, ConsumeRest, Context},
    types::Message,
};
use sqlx::PgPool;

mod config;

#[derive(Debug)]
pub enum Error {
    RevoltError(RevoltError),
}

impl From<RevoltError> for Error {
    fn from(value: RevoltError) -> Self {
        Self::RevoltError(value)
    }
}

struct CommandEvents;
#[async_trait]
impl CommandEventHandler<Error, State> for CommandEvents {}

struct Events(CommandHandler<CommandEvents, Error, State>);

#[async_trait]
impl EventHandler<Error> for Events {
    async fn message(&self, context: &MessageContext<'_>, message: Message) -> Result<(), Error> {
        println!("{message:?}");

        self.0.process_commands(context, message).await
    }
}

#[command("hello", error = Error, state = State)]
async fn add(ctx: &Context<'_, Error, State>, def: ConsumeRest) -> Result<(), Error> {
    // TODO

    Ok(())
}

#[derive(Clone)]
struct State {
    pub pool: PgPool,
}

#[tokio::main]
async fn main() {
    let config =
        toml::from_str::<config::Config>(&std::fs::read_to_string("Highlight.toml").unwrap())
            .unwrap();

    let pool = PgPool::connect(&config.database.url).await.unwrap();

    let state = State { pool };

    let commands = CommandHandler::new(CommandEvents, state)
        .with_static_prefix(&config.bot.prefix)
        .register(commands![add]);

    let client = Client::new(Events(commands), &config.revolt.api).await;

    client.run(&config.bot.token).await;
}
