use std::fmt::Debug;

use async_trait::async_trait;

use crate::{commands::Context, Error};

#[async_trait]
#[allow(unused)]
pub trait CommandEventHandler<E: From<Error> + Debug + Send + Sync + 'static, S: Debug + Clone + Send + Sync> {
    async fn command(&self, context: &mut Context<'_, E, S>) -> Result<(), E> {
        Ok(())
    }

    async fn no_command(&self, context: &mut Context<'_, E, S>) -> Result<(), E> {
        Ok(())
    }

    async fn error(&self, context: &mut Context<'_, E, S>, error: E) -> Result<(), E> {
        println!("Error: {error:?}");

        Ok(())
    }
}
