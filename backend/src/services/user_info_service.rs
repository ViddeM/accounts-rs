use rocket::State;
use sqlx::Pool;

use crate::{
    db::{account_repository, login_details_repository, new_transaction, DB},
    models::account::Account,
    util::{accounts_error::AccountsError, uuid::uuid_to_sqlx},
};

use super::{oauth_authorization_service::AccessToken, redis_service::RedisError};

#[derive(Debug, thiserror::Error)]
pub enum UserInfoError {
    #[error("Redis error")]
    RedisError(#[from] RedisError),
    #[error("")]
    InvalidAccessToken,
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("Account not found")]
    AccountNotFound,
    #[error("Login details not found")]
    LoginDetailsNotFound,
    #[error("Accounts error")]
    AccountsError(#[from] AccountsError),
}

#[derive(Debug)]
pub struct UserInfo {
    pub account: Account,
    pub email: String,
}

pub async fn get_user_info(
    db_pool: &State<Pool<DB>>,
    access_token: AccessToken,
) -> Result<UserInfo, UserInfoError> {
    let mut transaction = new_transaction(db_pool).await?;

    let account =
        account_repository::get_account(&mut transaction, uuid_to_sqlx(access_token.account_id))
            .await?
            .ok_or(UserInfoError::AccountNotFound)?;

    let login_details = login_details_repository::get_by_account_id(
        &mut transaction,
        uuid_to_sqlx(access_token.account_id),
    )
    .await?
    .ok_or(UserInfoError::LoginDetailsNotFound)?;

    Ok(UserInfo {
        account,
        email: login_details.email,
    })
}
