use sqlx::types::chrono::{DateTime, Utc};

pub const LOCAL_LOGIN_PROVIDER: &str = "local";

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginProvider {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
