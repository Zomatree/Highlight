use stoat::{Client, commands::CommandHandler};

mod commands;
mod events;
mod utils;

pub use utils::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let state = State::new().await;

    state.ensure_db().await;

    let commands = CommandHandler::new(commands::CommandEvents, state.clone())
        .help_command(Some(HighlightHelpCommand))
        .register(commands::commands());

    let events = events::Events {
        commands,
        state: state.clone(),
    };

    Client::new_with_api_url(events, &state.config.stoat.api)
        .await?
        .run(&state.config.bot.token)
        .await
}
