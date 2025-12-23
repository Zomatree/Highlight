use stoat::{Client, Context, EventHandler, async_trait};

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
struct Events;

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
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Client::new(Events).await?
        .run("TOKEN HERE")
        .await
}
