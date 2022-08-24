use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthorizationCode {
    pub id: Uuid,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
