use sqlx::FromRow;

#[derive(FromRow)]
pub struct Highlight {
    pub user_id: String,
    pub server_id: String,
    pub keyword: String,
}
