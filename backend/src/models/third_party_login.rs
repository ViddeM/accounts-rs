use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ThirdPartyLogin {
    pub account_id: Uuid,
    pub login_provider_id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
