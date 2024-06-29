use crate::api::auth::session_guard::Session;
use crate::api::oauth::authorize::rocket_uri_macro_get_authorization;
use crate::db::DB;
use crate::models::oauth_scope::OauthScopes;
use crate::services::oauth_consent_service;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::State;

#[derive(Debug, Clone, FromForm)]
pub struct ConsentForm {
    scopes: String,
    client_id: String,
    state: String,
    response_type: String,
    redirect_uri: String,
    accept: Option<String>,
    deny: Option<String>,
}

#[derive(Responder)]
pub enum PostConsentResponse {
    #[response(status = 500)]
    InternalError(String),
    #[response(status = 422)]
    FormError(String),
    ConsentDenied(Redirect),
    ConsentGranted(Redirect),
}

#[post("/consent", data = "<data>")]
pub async fn post_consent(
    db_pool: &State<sqlx::Pool<DB>>,
    data: Form<ConsentForm>,
    session: Session,
) -> PostConsentResponse {
    let accept = match (&data.accept, &data.deny) {
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (accept, deny) => {
            log::error!("Got invalid accept/deny combination from consent form, accept: {}, deny: {}. Since these values are filled in by this service they have either been tampered with by the user or we have a bug!", accept.is_some(), deny.is_some());
            return PostConsentResponse::FormError("Invalid accept/deny combination".to_string());
        }
    };

    if !accept {
        // The user decided to decline to approve this client.
        // TODO: we probably wanna send an error as a query parameter.
        return PostConsentResponse::ConsentDenied(Redirect::found(data.redirect_uri.clone()));
    }

    let requested_scopes = match OauthScopes::parse(&data.scopes) {
        Ok(s) => s.scopes,
        Err(err) => {
            log::error!("Failed to parse requested scopes, this should have been handled earlier, either we have a bug or someone is tampering with the form, err: {err:?}");
            return PostConsentResponse::FormError("Invalid scopes provided".to_string());
        }
    };

    if let Err(err) = oauth_consent_service::consent_to_client_scopes(
        db_pool,
        &data.client_id,
        session.account_id,
        requested_scopes,
    )
    .await
    {
        match err {
            oauth_consent_service::ConsentError::NoClientWithId => {
                log::error!("No client with the provided ID exists, either we have a bug or someone tampered with the form. ClientID: {}", data.client_id);
                return PostConsentResponse::FormError("Invalid client ID".to_string());
            }
            oauth_consent_service::ConsentError::ClientNotRegisteredForScope => {
                log::error!("The client was not registered for the requested scope, however, this should have been cleared earlier so either we have a bug or someone tampered with the form, client: {}, scopes: {}", data.client_id, data.scopes);
                return PostConsentResponse::FormError("The client was not registered for these scopes and they therefore cannot be requested for it.".to_string());
            }
            err => {
                log::error!(
                    "An unexpected error occurred whilst consenting to client, err: {err:?}"
                );
                return PostConsentResponse::InternalError(
                    "An unexpected error occurred".to_string(),
                );
            }
        }
    }

    let get_authorization_uri = format!(
        "/api/oauth/{}",
        uri!(get_authorization(
            &data.response_type,
            &data.client_id,
            &data.redirect_uri,
            &data.state,
            Some(&data.scopes)
        ))
        .to_string()
    );
    PostConsentResponse::ConsentGranted(Redirect::found(get_authorization_uri))
}
