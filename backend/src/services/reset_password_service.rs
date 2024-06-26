use chrono::Utc;
use mobc_redis::RedisConnectionManager;
use sqlx::types::uuid::Uuid;
use sqlx::Pool;

use crate::{
    api::auth::session_guard,
    db::{login_details_repository, new_transaction, reset_password_repository, DB},
    models::password_reset::PasswordReset,
    services::{email_service::EmailError, password_service},
    util::{accounts_error::AccountsError, config::Config, uuid::uuid_from_sqlx},
};

use super::email_service::EmailProvider;

#[derive(Debug, thiserror::Error)]
pub enum ResetPasswordError {
    #[error("An internal error occured")]
    Internal, // An internal error occurred

    #[error("Email error")]
    EmailError(#[from] EmailError),

    #[error("Invalid email or code")]
    InvalidEmailOrCode, // Either there is no account for that email or the code was invalid, or no password reset has been requested for that email.
}

impl From<AccountsError> for ResetPasswordError {
    fn from(_: AccountsError) -> Self {
        ResetPasswordError::Internal
    }
}

impl From<sqlx::Error> for ResetPasswordError {
    fn from(_: sqlx::Error) -> Self {
        ResetPasswordError::Internal
    }
}

// 1 minute
const RESET_PASSWORD_COOLDOWN_SECONDS: i64 = 60;

pub async fn initiate_password_reset(
    email_provider: &EmailProvider,
    db_pool: &Pool<DB>,
    email: String,
) -> Result<(), ResetPasswordError> {
    let mut transaction = new_transaction(db_pool).await?;

    let existing_with_email = login_details_repository::get_by_email(&mut transaction, &email)
        .await
        .map_err(|e| {
            error!("DB err: {:?}", e);
            ResetPasswordError::Internal
        })?;

    let account = match existing_with_email {
        Some(s) => s,
        None => {
            println!(
                "Tried to reset password for email '{}' with no account",
                email
            );
            return Ok(());
        }
    };

    let existing_password_reset =
        reset_password_repository::get_by_login_details(&mut transaction, account.account_id)
            .await
            .map_err(|err| {
                error!(
                    "Failed to retrieve existing password reset for account, err: {:?}",
                    err
                );
                ResetPasswordError::Internal
            })?;

    if let Some(password_reset) = existing_password_reset {
        let now = Utc::now();
        let diff = now.signed_duration_since(password_reset.created_at);
        if diff.num_seconds() < RESET_PASSWORD_COOLDOWN_SECONDS {
            // Less than the required number of seconds has passed since the last password
            println!("Tried to reset password within cooldown period");
            return Ok(());
        } else {
            // Delete the old password reset and create a new one
            reset_password_repository::delete_password_reset(&mut transaction, password_reset.id)
                .await
                .map_err(|err| {
                    error!("Failed to delete old password reset, err: {:?}", err);
                    ResetPasswordError::Internal
                })?;
        }
    }

    let reset_password = reset_password_repository::insert(&mut transaction, account.account_id)
        .await
        .map_err(|err| {
            error!("Failed to create reset_password, err {:?}", err);
            ResetPasswordError::Internal
        })?;

    let email_content = format_reset_password_email_content(&reset_password);

    email_provider
        .send_email(
            &account.email,
            "Password reset request for your accounts-rs account",
            &email_content,
        )
        .await?;

    transaction.commit().await?;

    Ok(())
}

fn format_reset_password_email_content(password_reset: &PasswordReset) -> String {
    format!(
        r#"Hi!
        
A request has been made to reset the password of your accounts-rs account.
If you requested this password reset, here is your reset code: 

{code}

This code will be valid for 3 hours.

If you did not request this password reset you can safely ignore this email.
        "#,
        code = password_reset.code
    )
}

pub async fn update_password(
    config: &Config,
    db_pool: &Pool<DB>,
    redis_pool: &mobc::Pool<RedisConnectionManager>,
    email: String,
    code: Uuid,
    password: String,
) -> Result<(), ResetPasswordError> {
    let mut transaction = new_transaction(db_pool).await?;

    let account = login_details_repository::get_by_email(&mut transaction, &email)
        .await
        .map_err(|e| {
            error!("DB err: {:?}", e);
            ResetPasswordError::Internal
        })?
        .ok_or(ResetPasswordError::InvalidEmailOrCode)?;

    let password_reset_code = reset_password_repository::get_by_login_details_and_code(
        &mut transaction,
        account.account_id,
        code,
    )
    .await?
    .ok_or(ResetPasswordError::InvalidEmailOrCode)?;

    let (hashed_password, nonces) =
        password_service::hash_and_encrypt_password(password.to_owned(), config).map_err(|e| {
            error!("Failed to hash and encrypt password: {:?}", e);
            ResetPasswordError::Internal
        })?;

    login_details_repository::update_account_password(
        &mut transaction,
        &account,
        &hashed_password,
        &nonces,
    )
    .await
    .map_err(|err| {
        error!(
            "Failed to update password for login details (account id: {}), err {:?}",
            account.account_id, err
        );
        ResetPasswordError::Internal
    })?;

    reset_password_repository::delete_password_reset(&mut transaction, password_reset_code.id)
        .await?;

    session_guard::reset_account_sessions(redis_pool, uuid_from_sqlx(account.account_id))
        .await
        .map_err(|err| {
            error!("Failed to reset account sessions, err: {}", err);
            ResetPasswordError::Internal
        })?;

    transaction.commit().await?;
    Ok(())
}
