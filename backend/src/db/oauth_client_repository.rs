use sqlx::{types::Uuid, Transaction};

use crate::{models::oauth_client::OauthClient, util::accounts_error::AccountsResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    client_id: String,
    client_secret: String,
    client_name: String,
    redirect_uri: String,
) -> AccountsResult<OauthClient> {
    Ok(sqlx::query_as!(
        OauthClient,
        "
INSERT INTO oauth_client (client_id, client_secret, client_name, redirect_uri)
VALUES                   ($1,        $2,            $3,          $4)
RETURNING *
        ",
        client_id,
        client_secret,
        client_name,
        redirect_uri
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn get_all(transaction: &mut Transaction<'_, DB>) -> AccountsResult<Vec<OauthClient>> {
    Ok(sqlx::query_as!(
        OauthClient,
        "
SELECT *
FROM oauth_client
        "
    )
    .fetch_all(transaction)
    .await?)
}

pub async fn get_by_client_name(
    transaction: &mut Transaction<'_, DB>,
    client_name: String,
) -> AccountsResult<Option<OauthClient>> {
    Ok(sqlx::query_as!(
        OauthClient,
        "
SELECT *
FROM oauth_client
WHERE client_name=$1
        ",
        client_name
    )
    .fetch_optional(transaction)
    .await?)
}

pub async fn delete_by_id(
    transaction: &mut Transaction<'_, DB>,
    id: Uuid,
) -> AccountsResult<Option<OauthClient>> {
    Ok(sqlx::query_as!(
        OauthClient,
        "
DELETE
FROM oauth_client
WHERE id=$1
RETURNING *
        ",
        id
    )
    .fetch_optional(transaction)
    .await?)
}
