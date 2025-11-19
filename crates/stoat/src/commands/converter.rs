use std::sync::LazyLock;

use async_trait::async_trait;
use regex::Regex;
use stoat_models::v0::{Channel, Member, Role, User};

use crate::{Error, commands::Context};

static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})$").unwrap());
static USER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^<@([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$").unwrap());
static CHANNEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^<#([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$").unwrap());
static ROLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^<%([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$").unwrap());

#[async_trait]
pub trait Converter<E: From<Error>, S: Send + Sync>: Sized {
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        let input = context.words.next().ok_or(Error::MissingParameter)?;

        Self::convert(context, input).await
    }

    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E>;
}

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for u32 {
    async fn convert(_context: &Context<E, S>, input: String) -> Result<Self, E> {
        input
            .parse::<u32>()
            .map_err(|e| Error::ConverterError(e.to_string()).into())
    }
}

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for String {
    async fn convert(_context: &Context<E, S>, input: String) -> Result<Self, E> {
        Ok(input)
    }
}

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for User {
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Some(captures) = USER_REGEX
            .captures(&input)
            .or_else(|| ID_REGEX.captures(&input))
        {
            let id = captures.get(1).unwrap().as_str();

            let user = context.cache.get_user(id).await;

            if let Some(user) = user {
                return Ok(user.clone());
            } else if let Ok(user) = context.http.fetch_user(id).await {
                return Ok(user);
            };
        };

        Err(Error::ConverterError("User not found".to_string()).into())
    }
}

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for Channel {
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Some(captures) = CHANNEL_REGEX
            .captures(&input)
            .or_else(|| ID_REGEX.captures(&input))
        {
            let id = captures.get(1).unwrap().as_str();

            if let Some(channel) = context.cache.get_channel(id).await {
                return Ok(channel);
            }
        };

        Err(Error::ConverterError("Channel not found".to_string()).into())
    }
}

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for Role {
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        let Some(server) = context.get_current_server().await else {
            return Err(Error::ConverterError("Role not found".to_string()).into());
        };

        if let Some(captures) = ROLE_REGEX
            .captures(&input)
            .or_else(|| ID_REGEX.captures(&input))
        {
            let id = captures.get(1).unwrap().as_str();

            if let Some(role) = server.roles.get(id) {
                return Ok(role.clone());
            }
        };

        Err(Error::ConverterError("Role not found".to_string()).into())
    }
}

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for Member {
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Some(server) = context.get_current_server().await {
            let user = <User as Converter<E, S>>::convert(context, input).await?;

            if let Some(member) = context.cache.get_member(&server.id, &user.id).await {
                return Ok(member);
            } else if let Ok(member) = context.http.fetch_member(&server.id, &user.id).await {
                return Ok(member);
            };
        };

        Err(Error::ConverterError("Member not found".to_string()).into())
    }
}

pub struct ConsumeRest(pub String);

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for ConsumeRest {
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        let words = context.words.rest();

        Ok(Self(words.join(" ")))
    }

    async fn convert(_context: &Context<E, S>, _input: String) -> Result<Self, E> {
        unreachable!()
    }
}

pub struct Rest(pub Vec<String>);

#[async_trait]
impl<E: From<Error>, S: Send + Sync> Converter<E, S> for Rest {
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        Ok(Self(context.words.rest()))
    }

    async fn convert(_context: &Context<E, S>, _input: String) -> Result<Self, E> {
        unreachable!()
    }
}
