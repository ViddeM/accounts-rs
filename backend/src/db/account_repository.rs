use crate::db::DB;
use crate::models::account::Account;
use crate::models::authority::AuthorityLevel;
use crate::util::accounts_error::AccountsResult;
use sqlx::types::uuid::Uuid;
use sqlx::Transaction;

pub async fn insert(
    transaction: &mut Transaction<'_, DB>,
    first_name: &str,
    last_name: &str,
) -> AccountsResult<Account> {
    Ok(sqlx::query_as!(
        Account,
        r#"
INSERT INTO account (first_name, last_name)
VALUES              ($1,         $2       )
RETURNING id, first_name, last_name, created_at, modified_at, authority as "authority: _" 
        "#,
        first_name,
        last_name,
    )
    .fetch_one(&mut **transaction)
    .await?)
}

pub async fn delete_multiple(
    transaction: &mut Transaction<'_, DB>,
    ids: &[Uuid],
) -> AccountsResult<()> {
    sqlx::query_as!(
        Account,
        "
DELETE 
FROM account
WHERE id = ANY($1)
    ",
        ids
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn get_admin_account(
    transaction: &mut Transaction<'_, DB>,
    account_id: Uuid,
) -> AccountsResult<Option<Account>> {
    Ok(sqlx::query_as!(
        Account,
        r#"
SELECT id, first_name, last_name, created_at, modified_at, authority AS "authority: _"
FROM account
WHERE id = $1 AND authority = $2
        "#,
        account_id,
        AuthorityLevel::Admin as _
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn get_account(
    transaction: &mut Transaction<'_, DB>,
    account_id: Uuid,
) -> AccountsResult<Option<Account>> {
    Ok(sqlx::query_as!(
        Account,
        r#"
SELECT id, first_name, last_name, created_at, modified_at, authority AS "authority: _"
FROM account
WHERE id = $1
        "#,
        account_id
    )
    .fetch_optional(&mut **transaction)
    .await?)
}

pub async fn get_all_accounts(
    transaction: &mut Transaction<'_, DB>,
) -> AccountsResult<Vec<Account>> {
    Ok(sqlx::query_as!(
        Account,
        r#"
SELECT id, first_name, last_name, created_at, modified_at, authority AS "authority: _"
FROM account
    "#
    )
    .fetch_all(&mut **transaction)
    .await?)
}
