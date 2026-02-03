mod config;
mod duration;
mod error;
mod help;
mod message;
mod models;
mod regex;
mod state;

pub use config::*;
pub use duration::*;
pub use error::*;
pub use help::*;
pub use message::*;
pub use models::*;
pub use regex::*;
pub use state::*;

pub type Command = stoat::commands::Command<Error, State>;
pub type CmdCtx = stoat::commands::Context<Error, State>;
pub type Result<T, E = Error> = std::result::Result<T, E>;
