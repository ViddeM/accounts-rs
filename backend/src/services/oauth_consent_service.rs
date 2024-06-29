use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    db::{
        client_scope_repository, new_transaction, oauth_client_repository,
        user_client_consent_repository, user_client_consent_scope_repository, DB,
    },
    models::{client_scope::ClientScope, oauth_scope::OauthScope},
    util::accounts_error::AccountsError,
};

use super::oauth_authorization_service::validate_scopes;

#[derive(Debug, thiserror::Error)]
pub enum ConsentError {
    #[error("Accounts error")]
    AccountsError(#[from] AccountsError),
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("There is no client with that client_id")]
    NoClientWithId,
    #[error("The client has not been registered for one or more of the requested scopes")]
    ClientNotRegisteredForScope,
}

pub async fn consent_to_client_scopes(
    db_pool: &sqlx::Pool<DB>,
    client_id: &String,
    account_id: Uuid,
    requested_scopes: HashSet<OauthScope>,
) -> Result<(), ConsentError> {
    let mut transaction = new_transaction(&db_pool).await?;

    let client = oauth_client_repository::get_by_client_id(&mut transaction, &client_id)
        .await?
        .ok_or(ConsentError::NoClientWithId)?;

    let scopes = client_scope_repository::get_all_for_client(&mut transaction, &client).await?;
    if !validate_scopes(&requested_scopes, &scopes) {
        return Err(ConsentError::ClientNotRegisteredForScope);
    }

    let consented_scopes: Vec<ClientScope> = scopes
        .into_iter()
        .filter(|s| requested_scopes.contains(&s.scope))
        .collect();

    let user_consent = if let Some(consent) =
        user_client_consent_repository::get_by_client_and_account(
            &mut transaction,
            &client,
            &account_id,
        )
        .await?
    {
        consent
    } else {
        user_client_consent_repository::insert(&mut transaction, &client, &account_id).await?
    };

    for scope in consented_scopes.into_iter() {
        user_client_consent_scope_repository::insert(&mut transaction, &user_consent, &scope)
            .await?;
    }

    transaction.commit().await?;

    Ok(())
}
