use revolt::{Client, commands::CommandHandler};

mod commands;
mod events;
mod utils;

pub use utils::*;

#[tokio::main]
async fn main() {
    let state = State::new().await;

    let commands = CommandHandler::new(commands::CommandEvents, state.clone())
        .with_static_prefix(&state.config.bot.prefix)
        .register(commands::commands());

    let events = events::Events {
        commands,
        state: state.clone()
    };

    let client = Client::new(events, &state.config.revolt.api).await;

    client.run(&state.config.bot.token).await;
}
