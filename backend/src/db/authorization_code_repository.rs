use sqlx::Transaction;

use crate::{models::authorization_code::AuthorizationCode, util::accounts_error::AccountsResult};

use super::DB;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    token: String,
) -> AccountsResult<AuthorizationCode> {
    Ok(sqlx::query_as!(
        AuthorizationCode,
        "
INSERT
INTO authorization_code(token)
VALUES                 ($1)
RETURNING *
    ",
        token
    )
    .fetch_one(transaction)
    .await?)
}
