use sqlx::FromRow;

#[derive(FromRow)]
pub struct ServerConfig {
    pub star_count: i32,
    pub starboard_channel: Option<String>,
}

#[derive(FromRow)]
pub struct OriginalMessage {
    #[sqlx(rename = "message_id")]
    pub id: String,
    #[sqlx(rename = "channel_id")]
    pub channel: String,
}
