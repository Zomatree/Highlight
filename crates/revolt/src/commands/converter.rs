use std::{sync::LazyLock, fmt::Debug};

use async_trait::async_trait;
use revolt_models::v0::User;
use regex::Regex;

use crate::{commands::Context, Error};

static ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})$").unwrap());
static USER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^<@([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$").unwrap());

#[async_trait]
pub trait Converter<E: From<Error> + Clone + Debug + Send + Sync, S: Debug + Clone + Send + Sync>: Sized {
    async fn convert(context: &mut Context<E, S>, input: String) -> Result<Self, E>;
}

#[async_trait]
impl<E: From<Error> + Clone + Debug + Send + Sync, S: Debug + Clone + Send + Sync> Converter<E, S> for u32 {
    async fn convert(context: &mut Context<E, S>, input: String) -> Result<Self, E> {
        input
            .parse::<u32>()
            .map_err(|e| Error::ConverterError(e.to_string()).into())
    }
}

#[async_trait]
impl<E: From<Error> + Clone + Debug + Send + Sync, S: Debug + Clone + Send + Sync> Converter<E, S> for String {
    async fn convert(context: &mut Context<E, S>, input: String) -> Result<Self, E> {
        Ok(input)
    }
}

#[async_trait]
impl<E: From<Error> + Clone + Debug + Send + Sync, S: Debug + Clone + Send + Sync> Converter<E, S> for User {
    async fn convert(context: &mut Context<E, S>, input: String) -> Result<Self, E> {
        if let Some(captures) = USER_REGEX.captures(&input).or_else(|| ID_REGEX.captures(&input)) {
            let id = captures.get(1).unwrap().as_str();

            let cache = context.cache.read().await;
            let user = cache.users.get(id).cloned();
            drop(cache);

            if let Some(user) = user {
                return Ok(user.clone())
            } else {
                let mut cache = context.cache.write().await;
                let user = context.http.get_user(id).await?;
                cache.users.insert(user.id.clone(), user.clone());

                return Ok(user)
            };
        };

        Err(Error::ConverterError("User not found".to_string()).into())
    }
}

pub struct ConsumeRest(pub String);

#[async_trait]
impl<E: From<Error> + Clone + Debug + Send + Sync, S: Debug + Clone + Send + Sync> Converter<E, S> for ConsumeRest {
    async fn convert(context: &mut Context<E, S>, input: String) -> Result<Self, E> {
        let mut output = input;

        let rest = context.words.rest().join(" ");

        if !rest.is_empty() {
            output.push(' ');
            output.push_str(&rest);
        };

        Ok(ConsumeRest(output))
    }
}
