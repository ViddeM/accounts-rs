use crate::api::core::login::rocket_uri_macro_get_login_page;
use crate::models::oauth_scope::OauthScopes;
use crate::{
    api::{
        auth::session_guard::Session,
        response::{ErrMsg, ResponseStatus},
    },
    db::DB,
    models::oauth_scope::ScopeField,
    services::oauth_authorization_service::{self, Oauth2Error},
};
use mobc_redis::RedisConnectionManager;
use rocket::{http::Status, response::Redirect, State};
use rocket_dyn_templates::Template;
use serde::Serialize;
use sqlx::Pool;

const CONSENT_TEMPLATE_NAME: &str = "oauth_consent";
const RESPONSE_TYPE_CODE: &str = "code";

#[derive(Serialize)]
pub struct ConsentTemplateFields {
    client_name: String,
    scope_fields: Vec<ScopeField>,
    scope: String,
    client_id: String,
    state: String,
    response_type: String,
    redirect_uri: String,
}

#[derive(Responder)]
pub enum GetAuthorizationResponse {
    #[response(status = 200)]
    Consent(Template),
    LoginRequired(Redirect),
    Failure(ResponseStatus<()>),
    Success(Redirect),
}

// TODO: Perhaps support nonce.
/// First step in the oauth2 authorization flow.
#[get("/authorize?<response_type>&<client_id>&<redirect_uri>&<state>&<scope>")]
pub async fn get_authorization(
    db_pool: &State<Pool<DB>>,
    redis_pool: &State<mobc::Pool<RedisConnectionManager>>,
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: String,
    scope: Option<String>,
    session: Option<Session>,
) -> GetAuthorizationResponse {
    if response_type != RESPONSE_TYPE_CODE {
        return GetAuthorizationResponse::Failure(ResponseStatus::err(
            Status::UnprocessableEntity,
            ErrMsg::InvalidResponseType,
        ));
    }

    let requested_scopes = match OauthScopes::parse_or_default(&scope) {
        Ok(s) => s.scopes,
        Err(err) => {
            log::warn!("Failed to parse requested scopes, err: {err:?}");
            return GetAuthorizationResponse::Failure(ResponseStatus::err(
                Status::UnprocessableEntity,
                ErrMsg::InvalidScope,
            ));
        }
    };

    // If the user is not currently logged in, redirect them to the login page then returning to this endpoint.
    let session = match session {
        Some(s) => s,
        None => {
            let return_to = format!(
                "/api/oauth/{}",
                uri!(get_authorization(
                    response_type,
                    client_id,
                    redirect_uri,
                    state,
                    scope,
                ))
                .to_string()
            );

            let login_uri = format!(
                "/api/core/{}",
                uri!(get_login_page(Some(return_to))).to_string()
            );

            return GetAuthorizationResponse::LoginRequired(Redirect::found(login_uri));
        }
    };

    let url = match oauth_authorization_service::get_auth_token(
        db_pool,
        redis_pool,
        client_id.clone(),
        &redirect_uri,
        &state,
        session.account_id,
        &requested_scopes,
    )
    .await
    {
        Ok(url) => url,
        Err(Oauth2Error::ScopeNotRegistered) => {
            error!("The client was not registered for one or more of the requested scopes");
            return GetAuthorizationResponse::Failure(ResponseStatus::err(
                Status::BadRequest,
                ErrMsg::InvalidScope,
            ));
        }
        Err(Oauth2Error::NoClientWithId) => {
            error!("No client with id '{}'", client_id);
            return GetAuthorizationResponse::Failure(ResponseStatus::err(
                Status::BadRequest,
                ErrMsg::InvalidClientId,
            ));
        }
        Err(Oauth2Error::InvalidRedirectUri) => {
            return GetAuthorizationResponse::Failure(ResponseStatus::err(
                Status::BadRequest,
                ErrMsg::InvalidRedirectUri,
            ))
        }
        Err(Oauth2Error::MissingClientConsent { client_name }) => {
            let scope_str = requested_scopes
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            let template_fields = ConsentTemplateFields {
                client_name,
                scope_fields: requested_scopes
                    .into_iter()
                    .map(|s| s.get_scope_field())
                    .collect(),
                scope: scope_str,
                client_id,
                state,
                response_type,
                redirect_uri,
            };

            return GetAuthorizationResponse::Consent(Template::render(
                CONSENT_TEMPLATE_NAME,
                template_fields,
            ));
        }
        Err(err) => {
            log::error!("An unexpected oauth2 error occurred, err: {err:?}");
            return GetAuthorizationResponse::Failure(ResponseStatus::internal_err());
        }
    };

    GetAuthorizationResponse::Success(Redirect::found(url))
}
