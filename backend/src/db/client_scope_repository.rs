use sqlx::Transaction;

use crate::{
    models::{
        client_scope::ClientScope, oauth_client::OauthClient,
        user_client_consent::UserClientConsent,
    },
    util::accounts_error::AccountsResult,
};

use super::DB;

pub async fn consented_by_user_for_client(
    transaction: &mut Transaction<'_, DB>,
    client: &OauthClient,
    user_consent: &UserClientConsent,
) -> AccountsResult<Vec<ClientScope>> {
    Ok(sqlx::query_as!(
        ClientScope,
        r#"
SELECT cs.id, cs.client_id, cs.scope as "scope: _", cs.created_at
FROM client_scope cs
JOIN user_client_consented_scope consented_scope ON consented_scope.client_scope_id = cs.id
WHERE consented_scope.user_client_consent_id = $1
AND cs.client_id = $2
        "#,
        user_consent.id,
        client.id,
    )
    .fetch_all(&mut **transaction)
    .await?)
}

pub async fn get_all_for_client(
    transaction: &mut Transaction<'_, DB>,
    client: &OauthClient,
) -> AccountsResult<Vec<ClientScope>> {
    Ok(sqlx::query_as!(
        ClientScope,
        r#"
SELECT id, client_id, scope as "scope: _", created_at
FROM client_scope
WHERE client_id = $1
        "#,
        client.id
    )
    .fetch_all(&mut **transaction)
    .await?)
}
