//! # Stoat API Wrapper
//!
//! A high-level Stoat API wrapper.
//!
//! ## Getting Started
//!
//! This crate requires some boilerplate to ensure errors are handled from the crate and your bot seemlessly
//! along with defining your event callbacks.
//!
//! ### Defining Your Error Type
//! Almost every event, command and callback uses your error type to allow you to propagate custom errors and
//! stoat-rs errors throughout your code.
//!
//! It is reconmended to use [`thiserror`](https://docs.rs/thiserror/latest/thiserror/) to define your error
//! however a manual implemenation is also possible if you prefer manually implementing [`From`].
//!
//! Your error must implement `From<stoat::Error>`, [`Debug`], [`Clone`], [`Send`], [`Sync`] and be `'static`.
//!
//! ```rust
//! #[derive(Debug, Clone, thiserror::Error)]
//! pub enum Error {
//!     #[error("Stoat Error: {0}")]
//!     StoatError(#[from] stoat::Error),
//! }
//! ```
//!
//! ### Setting Up Events
//! All events are implemented as a function in the [`EventHandler`] trait.
//!
//! Every event's first parameter is [`Context`] which contains the current bot state, this is given to you
//! in-place of your [`Client`].
//!
//! ```rust
//! use stoat::{async_trait, EventHandler, Context};
//!
//! #[derive(Debug, Clone)]
//! struct Events;
//!
//! #[async_trait]
//! impl EventHandler for Events {
//!     type Error = Error;  // Your error type defined above
//!
//!     async fn ready(&self, context: Context) -> Result<(), Self::Error> {
//!         println!("Ready!");
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ### Running The Bot
//! Once you define your events you can create your [`Client`] which will be the root entry for running your bot.
//!
//! For customizing your client's config see the [`Client`] documentation.
//!
//! ```rust
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     Client::new(Events).await?.run("BOT TOKEN").await
//! }
//! ```
//!
//! ## Commands
//!
//! See the [`commands`] module documentation for setting up commands.
//!
//! ## Logging
//! This crate makes use of the [`log`](https://docs.rs/log/latest/log/) crate to relay information and errors to you.
//!
//! Ensure you have a log implementation setup to see the logs.

#![doc(html_root_url = "https://docs.rs/stoat-rs/")]

pub mod builders;
pub mod cache;
pub mod client;
pub mod commands;
pub mod context;
pub mod error;
pub mod events;
pub mod ext;
pub mod file;
pub mod http;
pub mod notifiers;
pub mod permissions;
pub mod types;
pub mod ulid;
pub mod utils;
#[cfg(feature = "voice")]
pub mod voice;
pub mod websocket;

pub use cache::{CacheConfig, GlobalCache};
pub use client::Client;
pub use context::Context;
pub use error::{Error, Result};
pub use events::EventHandler;
pub use ext::*;
pub use file::LocalFile;
pub use http::HttpClient;
pub use ulid::Ulid;
pub use utils::*;
#[cfg(feature = "voice")]
pub use voice::*;

pub use async_trait::async_trait;

#[cfg(feature = "either")]
pub use either;
