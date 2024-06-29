use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::oauth_scope::OauthScope;

#[derive(Debug, Clone, PartialEq)]
pub struct ClientScope {
    pub id: Uuid,
    pub client_id: Uuid,
    pub scope: OauthScope,
    pub created_at: DateTime<Utc>,
}
