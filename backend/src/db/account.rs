use crate::accounts_error::AccountsResult;
use crate::models::account::Account;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> AccountRepository {
        AccountRepository { pool }
    }

    pub async fn get_by_email_and_password(
        &self,
        email: &String,
        password: &String,
    ) -> AccountsResult<Option<Account>> {
        Ok(sqlx::query_as!(
            Account,
            "
SELECT *
FROM account
WHERE email = $1
AND password = $2
        ",
            email,
            password
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn get_by_email(&self, email: &String) -> AccountsResult<Option<Account>> {
        Ok(sqlx::query_as!(
            Account,
            "\
SELECT * 
FROM account
WHERE email = $1
        ",
            email
        )
        .fetch_one(&self.pool)
        .await?)
    }
}
