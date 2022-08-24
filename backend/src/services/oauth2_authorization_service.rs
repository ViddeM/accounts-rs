use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::State;
use sqlx::Pool;

use crate::{
    db::{authorization_code_repository, new_transaction, oauth_client_repository, DB},
    util::accounts_error::AccountsError,
};

#[derive(Debug, thiserror::Error)]
pub enum Oauth2Error {
    #[error("An internal error occured")]
    Internal,
    #[error("There is no client with that client_id")]
    NoClientWithId,
    #[error("Redirect uri doesn't match client")]
    InvalidRedirectUri,
}

impl From<sqlx::Error> for Oauth2Error {
    fn from(_: sqlx::Error) -> Self {
        Oauth2Error::Internal
    }
}

impl From<AccountsError> for Oauth2Error {
    fn from(_: AccountsError) -> Self {
        Oauth2Error::Internal
    }
}

const AUTH_TOKEN_LENGTH: usize = 48;

pub async fn get_auth_token(
    db_pool: &State<Pool<DB>>,
    client_id: String,
    redirect_uri: String,
    state: String,
) -> Result<String, Oauth2Error> {
    let mut transaction = new_transaction(db_pool).await?;

    let client = oauth_client_repository::get_by_client_id(&mut transaction, client_id)
        .await?
        .ok_or(Oauth2Error::NoClientWithId)?;

    if redirect_uri != client.redirect_uri {
        error!(
            "Redirect uri doesn't match, request redirect_uri: {}, client set redirect_uri: {}",
            redirect_uri, client.redirect_uri
        );
        return Err(Oauth2Error::InvalidRedirectUri);
    }

    let auth_token = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH)
        .map(char::from)
        .collect();

    let auth_code = authorization_code_repository::insert(&mut transaction, auth_token).await?;

    Ok(String::new())
}
