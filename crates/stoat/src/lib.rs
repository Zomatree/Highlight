//! Stoat API Wrapper

#![doc(html_root_url = "https://docs.rs/stoat-rs/")]

pub mod builders;
pub mod cache;
pub mod client;
pub mod commands;
pub mod context;
pub mod error;
pub mod events;
pub mod ext;
pub mod http;
pub mod notifiers;
pub mod permissions;
pub mod types;
pub mod utils;
#[cfg(feature = "voice")]
pub mod voice;
pub mod websocket;

pub use cache::GlobalCache;
pub use client::Client;
pub use context::Context;
pub use error::{Error, Result};
pub use events::EventHandler;
pub use ext::*;
pub use http::HttpClient;
pub use utils::*;
#[cfg(feature = "voice")]
pub use voice::*;

pub use async_trait::async_trait;
