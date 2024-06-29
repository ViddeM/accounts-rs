use sqlx::Transaction;
use uuid::Uuid;

use crate::{
    models::{oauth_client::OauthClient, user_client_consent::UserClientConsent},
    util::accounts_error::AccountsResult,
};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    client: &OauthClient,
    account_id: &Uuid,
) -> AccountsResult<UserClientConsent> {
    Ok(sqlx::query_as!(
        UserClientConsent,
        " 
INSERT INTO user_client_consent (client_id, account_id)
VALUES                          ($1,        $2)
RETURNING id, client_id, account_id, consented_on
        ",
        client.id,
        account_id,
    )
    .fetch_one(&mut **transaction)
    .await?)
}

pub async fn get_by_client_and_account(
    transaction: &mut Transaction<'_, DB>,
    client: &OauthClient,
    account_id: &Uuid,
) -> AccountsResult<Option<UserClientConsent>> {
    Ok(sqlx::query_as!(
        UserClientConsent,
        "
SELECT id, client_id, account_id, consented_on
FROM user_client_consent
WHERE client_id = $1 AND account_id = $2
        ",
        client.id,
        account_id,
    )
    .fetch_optional(&mut **transaction)
    .await?)
}
