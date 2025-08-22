use std::fmt::Debug;

use async_trait::async_trait;

use crate::{commands::Context, Error};

#[async_trait]
pub trait Converter<E: From<Error> + Debug + Send + Sync, S: Debug + Clone + Send + Sync>: Sized {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E>;
}

#[async_trait]
impl<E: From<Error> + Debug + Send + Sync, S: Debug + Clone + Send + Sync> Converter<E, S> for u32 {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E> {
        input
            .parse::<u32>()
            .map_err(|e| Error::ConverterError(e.to_string()).into())
    }
}

#[async_trait]
impl<E: From<Error> + Debug + Send + Sync, S: Debug + Clone + Send + Sync> Converter<E, S> for String {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E> {
        Ok(input)
    }
}

pub struct ConsumeRest(pub String);

#[async_trait]
impl<E: From<Error> + Debug + Send + Sync, S: Debug + Clone + Send + Sync> Converter<E, S> for ConsumeRest {
    async fn convert(context: &mut Context<'_, E, S>, input: String) -> Result<Self, E> {
        let mut output = input;

        let rest = context.words.rest().join(" ");

        if !rest.is_empty() {
            output.push(' ');
            output.push_str(&rest);
        };

        Ok(ConsumeRest(output))
    }
}
