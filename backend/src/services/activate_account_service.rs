use crate::db::{activation_code_repository, login_details_repository, new_transaction, DB};
use crate::util::accounts_error::AccountsError;
use rocket::State;
use sqlx::Pool;
use uuid::Uuid;

pub enum ActivateAccountError {
    Internal,           // An internal error occurred
    NoAccountWithEmail, // No account exists with the provided email address
    InvalidCode,        // The provided code did not match the provided email address
}

impl From<AccountsError> for ActivateAccountError {
    fn from(_: AccountsError) -> Self {
        ActivateAccountError::Internal
    }
}

impl From<sqlx::Error> for ActivateAccountError {
    fn from(_: sqlx::Error) -> Self {
        ActivateAccountError::Internal
    }
}

pub async fn activate_account(
    email: &str,
    code: Uuid,
    db_pool: &State<Pool<DB>>,
) -> Result<(), ActivateAccountError> {
    let mut transaction = new_transaction(db_pool).await?;

    let login_details = login_details_repository::get_by_email(&mut transaction, email)
        .await?
        .ok_or(ActivateAccountError::NoAccountWithEmail)?;

    let activation_code_optional = activation_code_repository::get_by_login_details_and_code(
        &mut transaction,
        login_details.account_id,
        code,
    )
    .await?;

    let activation_code = activation_code_optional.ok_or(ActivateAccountError::InvalidCode)?;

    // The user has successfully activated their account, update the state to reflect that.
    login_details_repository::activate_account(&mut transaction, activation_code.login_details)
        .await?;

    activation_code_repository::delete(&mut transaction, activation_code.login_details).await?;

    transaction.commit().await?;

    Ok(())
}
