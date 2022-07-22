use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PasswordReset {
    pub id: Uuid,
    pub login_details: Uuid,
    pub code: Uuid,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
