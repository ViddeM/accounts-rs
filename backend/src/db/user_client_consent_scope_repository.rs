use sqlx::Transaction;

use crate::{
    models::{
        client_scope::ClientScope, user_client_consent::UserClientConsent,
        user_client_consent_scope::UserClientConsentedScope,
    },
    util::accounts_error::AccountsResult,
};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    client_consent: &UserClientConsent,
    client_scope: &ClientScope,
) -> AccountsResult<UserClientConsentedScope> {
    Ok(sqlx::query_as!(
        UserClientConsentedScope,
        "
INSERT INTO user_client_consented_scope(user_client_consent_id, client_scope_id)
VALUES                                 ($1,                     $2)
RETURNING id, user_client_consent_id, client_scope_id, created_at
        ",
        client_consent.id,
        client_scope.id
    )
    .fetch_one(&mut **transaction)
    .await?)
}
