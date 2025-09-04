use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub revolt: RevoltConfig,
    pub bot: BotConfig,
    pub database: DatabaseConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevoltConfig {
    pub api: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotConfig {
    pub prefix: String,
    pub token: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}
