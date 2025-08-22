use futures::future::BoxFuture;


pub mod command;
pub mod context;
pub mod converter;
pub mod events;
pub mod handler;
pub mod words;

pub use command::Command;
pub use context::Context;
pub use converter::*;
pub use events::CommandEventHandler;
pub use handler::CommandHandler;
pub use words::Words;

pub type CommandReturn<'a, E> = BoxFuture<'a, Result<(), E>>;