use sqlx::database::HasValueRef;
use sqlx::decode::Decode;
use sqlx::error::BoxDynError;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{Database, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Whitelist {
    pub email: String,
    // If None, assumed to refer to local login_details instead of
    pub login_provider_id: LoginProviderType,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum LoginProviderType {
    LoginProvider(String),
    LocalAccount,
}

impl<'r, DB: Database> Decode<'r, DB> for LoginProviderType
where
    Option<&'r str>: Decode<'r, DB>,
{
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let decoded_value = <Option<&str> as Decode<DB>>::decode(value)?;
        Ok(match decoded_value {
            None => LoginProviderType::LocalAccount,
            Some(id) => LoginProviderType::LoginProvider(id.to_string()),
        })
    }
}
