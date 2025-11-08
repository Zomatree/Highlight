use std::fmt::Debug;

use async_trait::async_trait;

use crate::{Error, commands::Context};

#[async_trait]
#[allow(unused)]
pub trait CommandEventHandler {
    type State: Debug + Clone + Send + Sync + 'static;
    type Error: From<Error> + Clone + Debug + Send + Sync + 'static;

    async fn command(&self, context: Context<Self::Error, Self::State>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn no_command(
        &self,
        context: Context<Self::Error, Self::State>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn after_command(
        &self,
        context: Context<Self::Error, Self::State>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn error(
        &self,
        context: Context<Self::Error, Self::State>,
        error: Self::Error,
    ) -> Result<(), Self::Error> {
        log::error!("{error:?}");

        Ok(())
    }
}
