use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub stoat: StoatConfig,
    pub bot: BotConfig,
    pub database: DatabaseConfig,
    pub limits: LimitsConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoatConfig {
    pub api: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotConfig {
    pub prefix: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LimitsConfig {
    pub max_keywords: usize,
    pub min_stars: i32,
}
