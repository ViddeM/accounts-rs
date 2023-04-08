use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rocket::State;
use sqlx::types::Uuid;

use crate::{
    db::{new_transaction, oauth_client_repository, DB},
    models::oauth_client::OauthClient,
    util::accounts_error::AccountsError,
};

const CLIENT_ID_LENGTH: usize = 32;
const CLIENT_SECRET_LENGTH: usize = 128;

#[derive(Debug, thiserror::Error)]
pub enum OauthClientError {
    #[error("An internal error occured")]
    Internal,
    #[error("The Oauth client name is already taken")]
    ClientNameTaken,
    #[error("The provided ID is not a valid UUID")]
    InvalidId,
    #[error("No client with that ID exists")]
    ClientIdNotFound,
}

impl From<sqlx::Error> for OauthClientError {
    fn from(_: sqlx::Error) -> Self {
        OauthClientError::Internal
    }
}

impl From<AccountsError> for OauthClientError {
    fn from(_: AccountsError) -> Self {
        OauthClientError::Internal
    }
}

pub async fn get_oauth_clients(
    db_pool: &State<sqlx::Pool<DB>>,
) -> Result<Vec<OauthClient>, OauthClientError> {
    let mut transaction = new_transaction(db_pool).await?;

    match oauth_client_repository::get_all(&mut transaction).await {
        Ok(clients) => Ok(clients),
        Err(err) => {
            error!("Failed to get oauth clients, err: {}", err);
            Err(err.into())
        }
    }
}

pub async fn create_oauth_client(
    db_pool: &State<sqlx::Pool<DB>>,
    client_name: String,
    redirect_uri: String,
) -> Result<OauthClient, OauthClientError> {
    let mut transaction = new_transaction(db_pool).await?;

    // Check if the name is taken already
    match oauth_client_repository::get_by_client_name(&mut transaction, client_name.clone()).await {
        Ok(None) => {}
        Ok(Some(_)) => return Err(OauthClientError::ClientNameTaken),
        Err(err) => {
            error!("Failed to get client by name, err: {}", err);
            Err(err)?;
        }
    };

    let client_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(CLIENT_ID_LENGTH)
        .map(char::from)
        .collect();

    let client_secret: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(CLIENT_SECRET_LENGTH)
        .map(char::from)
        .collect();

    let oauth_client = oauth_client_repository::insert(
        &mut transaction,
        client_id,
        client_secret,
        client_name,
        redirect_uri,
    )
    .await?;

    transaction.commit().await?;

    Ok(oauth_client)
}

pub async fn delete_oauth_client(
    db_pool: &State<sqlx::Pool<DB>>,
    id: String,
) -> Result<(), OauthClientError> {
    let id = Uuid::parse_str(&id).map_err(|err| {
        error!("Failed to parse oauth client id as UUID, err {}", err);
        OauthClientError::InvalidId
    })?;

    let mut transaction = new_transaction(db_pool).await?;
    oauth_client_repository::delete_by_id(&mut transaction, id)
        .await
        .map_err(|err| {
            error!("Failed to delete oauth client, err {}", err);
            OauthClientError::Internal
        })?
        .ok_or(OauthClientError::ClientIdNotFound)?;

    transaction.commit().await?;

    Ok(())
}
