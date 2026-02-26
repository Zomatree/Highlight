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

        let _ = context.events.close();

        Ok(())
    }
}
#[tokio::test]
async fn test_run() {
    let token = std::env::var("token").unwrap_or_else(|_| "token".to_string());
    let r = Client::new(Events).await.unwrap().run(token).await;
    assert!(r.is_ok());
}
