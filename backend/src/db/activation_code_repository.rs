use crate::db::DB;
use crate::models::activation_code::ActivationCode;
use crate::util::accounts_error::AccountsResult;
use sqlx::Transaction;
use uuid::Uuid;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    login_details: Uuid,
) -> AccountsResult<ActivationCode> {
    Ok(sqlx::query_as!(
        ActivationCode,
        "
INSERT INTO activation_code (login_details)
VALUES                      ($1           )
RETURNING id, login_details, code, created_at, modified_at
        ",
        login_details
    )
    .fetch_one(transaction)
    .await?)
}

pub async fn delete_outdated(
    transaction: &mut Transaction<'_, DB>,
    lifetime_minutes: u64,
) -> AccountsResult<Vec<ActivationCode>> {
    Ok(sqlx::query_as!(
        ActivationCode,
        "
DELETE
FROM activation_code
WHERE created_at + ($1 * interval '1 minute') > NOW()
RETURNING id, login_details, code, created_at, modified_at
        ",
        lifetime_minutes as f64
    )
    .fetch_all(transaction)
    .await?)
}
