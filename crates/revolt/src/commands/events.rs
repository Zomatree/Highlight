use std::fmt::Debug;

use async_trait::async_trait;

use crate::{Error, commands::Context};

#[async_trait]
#[allow(unused)]
pub trait CommandEventHandler<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>
{
    async fn command(&self, context: Context<E, S>) -> Result<(), E> {
        Ok(())
    }

    async fn no_command(&self, context: Context<E, S>) -> Result<(), E> {
        Ok(())
    }

    async fn error(&self, context: Context<E, S>, error: E) -> Result<(), E> {
        println!("Error: {error:?}");

        Ok(())
    }
}
