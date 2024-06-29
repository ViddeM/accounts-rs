use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserClientConsentedScope {
    pub id: Uuid,
    pub user_client_consent_id: Uuid,
    pub client_scope_id: Uuid,
    pub created_at: DateTime<Utc>,
}
