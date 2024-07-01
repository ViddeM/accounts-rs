use rocket::{http::Status, serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        auth::admin_session_guard::AdminSession,
        response::{EmptyResponse, ErrMsg, ResponseStatus},
    },
    db::DB,
    models::oauth_scope::OauthScope,
    services::oauth_client_service::{self, OauthClientError},
};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OauthClientsResponse {
    oauth_clients: Vec<OauthClientResponse>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OauthClientResponse {
    client_name: String,
    client_id: String,
    redirect_uri: String,
    id: String,
}

#[get("/oauth_clients")]
pub async fn get_oauth_clients(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
) -> ResponseStatus<OauthClientsResponse> {
    match oauth_client_service::get_oauth_clients(db_pool).await {
        Ok(clients) => ResponseStatus::ok(OauthClientsResponse {
            oauth_clients: clients
                .into_iter()
                .map(|client| OauthClientResponse {
                    client_name: client.client_name,
                    client_id: client.client_id,
                    redirect_uri: client.redirect_uri,
                    id: client.id.to_string(),
                })
                .collect(),
        }),
        Err(err) => {
            error!("Failed to get oauth_clients, err {}", err);
            ResponseStatus::internal_err()
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewClientRequest {
    pub client_name: String,
    pub redirect_uri: String,
    pub scopes: Vec<OauthScope>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewClientResponse {
    pub client_id: String,
    pub client_secret: String,
}

#[post("/oauth_clients", data = "<request>")]
pub async fn post_new_client(
    db_pool: &State<sqlx::Pool<DB>>,
    request: Json<NewClientRequest>,
    _admin_session: AdminSession,
) -> ResponseStatus<NewClientResponse> {
    match oauth_client_service::create_oauth_client(
        db_pool,
        &request.client_name,
        &request.redirect_uri,
        &request.scopes,
    )
    .await
    {
        Ok(client) => ResponseStatus::ok(NewClientResponse {
            client_id: client.client_id,
            client_secret: client.client_secret,
        }),
        Err(OauthClientError::ClientNameTaken) => {
            error!("The client name has already been taken");
            ResponseStatus::err(Status::UnprocessableEntity, ErrMsg::OauthClientNameTaken)
        }
        Err(err) => {
            error!("Failed to create oauth client, err: {}", err);
            ResponseStatus::internal_err()
        }
    }
}

#[delete("/oauth_clients/<id>")]
pub async fn delete_client(
    db_pool: &State<sqlx::Pool<DB>>,
    id: String,
    _admin_session: AdminSession,
) -> ResponseStatus<EmptyResponse> {
    match oauth_client_service::delete_oauth_client(db_pool, id).await {
        Ok(()) => ResponseStatus::<EmptyResponse>::ok_no_content(),
        Err(OauthClientError::InvalidId) => {
            ResponseStatus::err(Status::BadRequest, ErrMsg::InvalidUuid)
        }
        Err(OauthClientError::ClientIdNotFound) => {
            ResponseStatus::err(Status::NotFound, ErrMsg::InvalidClientId)
        }
        Err(_) => ResponseStatus::internal_err(),
    }
}
