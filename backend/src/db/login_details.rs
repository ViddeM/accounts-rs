use crate::models::login_details::LoginDetails;
use crate::util::accounts_error::AccountsResult;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct LoginDetailsRepository {
    pool: PgPool,
}

impl LoginDetailsRepository {
    pub fn new(pool: PgPool) -> LoginDetailsRepository {
        LoginDetailsRepository { pool }
    }

    pub async fn get_by_email_and_password(
        &self,
        email: &str,
        password: &str,
    ) -> AccountsResult<Option<LoginDetails>> {
        Ok(sqlx::query_as!(
            LoginDetails,
            "
SELECT *
FROM login_details
WHERE email = $1
AND password = $2
        ",
            email,
            password
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn get_by_email(&self, email: &str) -> AccountsResult<Option<LoginDetails>> {
        Ok(sqlx::query_as!(
            LoginDetails,
            "
SELECT *
FROM login_details
WHERE email = $1
        ",
            email
        )
        .fetch_optional(&self.pool)
        .await?)
    }
}
