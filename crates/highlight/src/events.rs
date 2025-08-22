use revolt::{Context, EventHandler, async_trait, commands::CommandHandler, types::Message};

use crate::{Error, State, commands::CommandEvents};

pub struct Events(pub CommandHandler<CommandEvents, Error, State>);

#[async_trait]
impl EventHandler<Error> for Events {
    async fn message(&self, context: &Context<'_>, message: Message) -> Result<(), Error> {
        println!("{message:?}");

        self.0.process_commands(context, message).await
    }

    async fn ready(&self, context: &Context<'_>) -> Result<(), Error> {
        println!("Ready!");

        Ok(())
    }
}
