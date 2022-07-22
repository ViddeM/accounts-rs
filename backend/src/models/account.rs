use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
