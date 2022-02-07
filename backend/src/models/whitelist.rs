use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Whitelist {
    pub email: String,
    pub login_provider: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
