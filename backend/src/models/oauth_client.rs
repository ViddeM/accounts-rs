use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OauthClient {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret: String,
    pub client_name: String,
    pub redirect_uri: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
