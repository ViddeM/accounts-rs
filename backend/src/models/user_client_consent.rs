use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserClientConsent {
    pub id: Uuid,
    pub client_id: Uuid,
    pub account_id: Uuid,
    pub consented_on: DateTime<Utc>,
}
