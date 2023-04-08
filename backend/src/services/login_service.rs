use chrono::{DateTime, Duration, Utc};
use rocket::State;
use sqlx::Pool;

use crate::{
    db::{login_details_repository, new_transaction, DB},
    models::login_details::LoginDetails,
    util::{accounts_error::AccountsError, config::Config},
};

use super::password_service;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("An internal error occurred")]
    Internal,
    #[error("Invalid email/password")]
    InvalidEmailPassword,
    #[error("Account is locked due to excessive incorrect login attempts")]
    AccountLocked,
    #[error("Account has not yet been activated")]
    AccountNotActivated,
}

impl From<AccountsError> for LoginError {
    fn from(_: AccountsError) -> Self {
        LoginError::Internal
    }
}
impl From<sqlx::Error> for LoginError {
    fn from(_: sqlx::Error) -> Self {
        LoginError::Internal
    }
}

pub async fn validate_login(
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
    email: String,
    password: String,
) -> Result<LoginDetails, LoginError> {
    let mut transaction = new_transaction(db_pool).await?;

    let login_details =
        match login_details_repository::get_by_email(&mut transaction, &email).await? {
            None => return Err(LoginError::InvalidEmailPassword),
            Some(l) => l,
        };

    if !password_service::verify_password(
        password.to_owned(),
        login_details.password.to_owned(),
        login_details.password_nonces.to_owned(),
        config,
    ) {
        // Password incorrect:
        //  - Increase the invalid password count
        //  - Set an appropriate account lockout depending on the number of incorrect password count.
        let new_invalid_password_count = login_details.incorrect_password_count + 1;
        let account_lockout_until = get_account_lockout(new_invalid_password_count);
        login_details_repository::set_account_lockout(
            &mut transaction,
            login_details.account_id,
            new_invalid_password_count,
            account_lockout_until,
        )
        .await?;

        transaction.commit().await?;
        return Err(LoginError::InvalidEmailPassword);
    }

    let now = Utc::now();
    if let Some(locked_until) = login_details.account_locked_until {
        if locked_until > now {
            return Err(LoginError::AccountLocked);
        }
    }

    // If we reach here, we now accept the user as authorized with the account

    if login_details.activated_at.is_none() {
        return Err(LoginError::AccountNotActivated);
    }

    // Reset account lock on successful login
    login_details_repository::set_account_lockout(
        &mut transaction,
        login_details.account_id,
        0,
        Option::None,
    )
    .await?;

    transaction.commit().await?;
    Ok(login_details)
}

/// Generate a time until which the account will be locked
/// Based on the given number of incorrect password count.
///
/// Returns an option which if Some(v) will contain the datetime until which the account should be locked
/// Or None if the account shouldn't be locked.
fn get_account_lockout(invalid_password_count: i32) -> Option<DateTime<Utc>> {
    let lockout_duration = match invalid_password_count {
        // We don't need to lock the account on the first couple of failures.
        0 => return None,
        1 => Duration::seconds(1),
        2 => Duration::seconds(5),
        3 => Duration::minutes(2),
        4 => Duration::minutes(15),
        5 => Duration::minutes(30),
        6 => Duration::hours(2),
        7 => Duration::days(1),
        8 => Duration::weeks(1),
        9 => Duration::weeks(2),
        _ => Duration::days(365),
    };

    let now = Utc::now();
    let locked_until = now
        .checked_add_signed(lockout_duration)
        .unwrap_or(DateTime::<Utc>::MAX_UTC);
    Some(locked_until)
}
