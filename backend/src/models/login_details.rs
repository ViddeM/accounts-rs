use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginDetails {
    pub account_id: Uuid,
    pub email: String,
    pub password: String,
    pub password_nonces: String,
    pub activated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub incorrect_password_count: i32,
    pub account_locked_until: Option<DateTime<Utc>>,
}
