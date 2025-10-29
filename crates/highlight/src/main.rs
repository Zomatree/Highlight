use stoat::{Client, commands::CommandHandler};

mod commands;
mod events;
mod utils;

pub use utils::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let state = State::new().await;

    state.ensure_db().await;

    let commands = CommandHandler::new(commands::CommandEvents, state.clone())
        .with_static_prefixes(vec![
            format!("<@{}> ", &state.config.bot.id),
            state.config.bot.prefix.clone(),
        ])
        .register(commands::commands());

    let events = events::Events {
        commands,
        state: state.clone(),
    };

    let client = Client::new(events, &state.config.stoat.api).await;

    client.run(&state.config.bot.token).await.unwrap();
}
