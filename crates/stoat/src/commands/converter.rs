use std::{fmt::Debug, sync::LazyLock};

use async_trait::async_trait;
use regex::Regex;
use stoat_models::v0::{Channel, Emoji, Member, Role, User};

use crate::{Error, commands::Context};

static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})$").unwrap());
static USER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^<@([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$").unwrap());
static CHANNEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^<#([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$").unwrap());
static ROLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^<%([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$").unwrap());
static EMOJI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^:([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26}):$").unwrap());

#[async_trait]
pub trait Converter<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
>: Sized
{
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        let input = context.words.next().ok_or(Error::MissingParameter)?;

        Self::convert(context, input).await
    }

    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E>;
}

macro_rules! impl_parse_converter {
    ($ty:ty) => {
        #[async_trait]
        impl<
            E: From<Error> + Clone + Debug + Send + Sync + 'static,
            S: Debug + Clone + Send + Sync + 'static,
        > Converter<E, S> for $ty
        {
            async fn convert(_context: &Context<E, S>, input: String) -> Result<Self, E> {
                input
                    .parse::<$ty>()
                    .map_err(|e| Error::ConverterError(e.to_string()).into())
            }
        }
    };
}

impl_parse_converter!(u8);
impl_parse_converter!(u16);
impl_parse_converter!(u32);
impl_parse_converter!(u64);
impl_parse_converter!(u128);
impl_parse_converter!(i8);
impl_parse_converter!(i16);
impl_parse_converter!(i32);
impl_parse_converter!(i64);
impl_parse_converter!(i128);
impl_parse_converter!(f32);
impl_parse_converter!(f64);

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for String
{
    async fn convert(_context: &Context<E, S>, input: String) -> Result<Self, E> {
        Ok(input)
    }
}

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for bool
{
    async fn convert(_context: &Context<E, S>, input: String) -> Result<Self, E> {
        match input.to_lowercase().as_str() {
            "yes" | "y" | "true" | "t" | "1" | "enable" | "enabled" | "on" => Ok(true),
            "no" | "n" | "false" | "f" | "0" | "disable" | "disabled" | "off" => Ok(false),
            _ => Err(Error::ConverterError("Bad boolean value".to_string()).into()),
        }
    }
}

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for User
{
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Some(captures) = USER_REGEX
            .captures(&input)
            .or_else(|| ID_REGEX.captures(&input))
        {
            let id = captures.get(1).unwrap().as_str();

            let user = context.cache.get_user(id);

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
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for Channel
{
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Some(captures) = CHANNEL_REGEX
            .captures(&input)
            .or_else(|| ID_REGEX.captures(&input))
        {
            let id = captures.get(1).unwrap().as_str();

            if let Some(channel) = context.cache.get_channel(id) {
                return Ok(channel);
            }
        };

        Err(Error::ConverterError("Channel not found".to_string()).into())
    }
}

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for Role
{
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        let Ok(server) = context.get_current_server() else {
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
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for Member
{
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Ok(server) = context.get_current_server() {
            let user = <User as Converter<E, S>>::convert(context, input).await?;

            if let Some(member) = context.cache.get_member(&server.id, &user.id) {
                return Ok(member);
            } else if let Ok(member) = context.http.fetch_member(&server.id, &user.id).await {
                return Ok(member);
            };
        };

        Err(Error::ConverterError("Member not found".to_string()).into())
    }
}

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for Emoji
{
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Some(captures) = EMOJI_REGEX
            .captures(&input)
            .or_else(|| ID_REGEX.captures(&input))
        {
            let id = captures.get(1).unwrap().as_str();

            if let Some(emoji) = context.cache.get_emoji(id) {
                return Ok(emoji);
            }
        } else {
            if let Some(emoji) = context
                .cache
                .emojis
                .any_sync(|_, emoji| &emoji.name == &input)
            {
                return Ok(emoji.get().clone());
            }
        };

        Err(Error::ConverterError("Emoji not found".to_string()).into())
    }
}

pub struct ConsumeRest(pub String);

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
> Converter<E, S> for ConsumeRest
{
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        let words = context.words.rest();

        Ok(Self(words.join(" ")))
    }

    async fn convert(_context: &Context<E, S>, _input: String) -> Result<Self, E> {
        unreachable!()
    }
}

#[cfg(feature = "either")]
#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    L: Converter<E, S>,
    R: Converter<E, S>,
> Converter<E, S> for either::Either<L, R>
{
    async fn convert(context: &Context<E, S>, input: String) -> Result<Self, E> {
        if let Ok(left) = L::convert(context, input.clone()).await {
            Ok(either::Either::Left(left))
        } else {
            R::convert(context, input).await.map(either::Either::Right)
        }
    }
}

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    T: Converter<E, S>,
> Converter<E, S> for Option<T>
{
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        let Some(input) = context.words.next() else {
            return Ok(None);
        };

        Ok(T::convert(context, input).await.ok())
    }

    async fn convert(_context: &Context<E, S>, _input: String) -> Result<Self, E> {
        unreachable!()
    }
}

pub struct Greedy<T>(pub Vec<T>);

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    T: Converter<E, S> + Send + Sync,
> Converter<E, S> for Greedy<T>
{
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        let mut converted = Vec::new();

        while let Some(arg) = context.words.next() {
            if let Ok(value) = T::convert(context, arg).await {
                converted.push(value);
            } else {
                context.words.undo();
                break;
            }
        }

        Ok(Self(converted))
    }

    async fn convert(_context: &Context<E, S>, _input: String) -> Result<Self, E> {
        unreachable!()
    }
}

#[async_trait]
impl<
    E: From<Error> + Clone + Debug + Send + Sync + 'static,
    S: Debug + Clone + Send + Sync + 'static,
    T: Converter<E, S> + Send + Sync,
> Converter<E, S> for Vec<T>
{
    async fn from_context(context: &Context<E, S>) -> Result<Self, E> {
        let mut converted = Vec::new();

        while let Some(arg) = context.words.next() {
            converted.push(T::convert(context, arg).await?);
        }

        Ok(converted)
    }

    async fn convert(_context: &Context<E, S>, _input: String) -> Result<Self, E> {
        unreachable!()
    }
}
