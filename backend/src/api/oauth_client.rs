use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{
    api::response::AccountsResponse,
    db::DB,
    services::{
        admin_session_service::AdminSession,
        oauth_client_service::{self, OauthClientError},
    },
};

use super::response::ApiError;

#[derive(Serialize)]
pub struct OauthClientsResponse {
    oauth_clients: Vec<OauthClientResponse>,
}

#[derive(Serialize)]
pub struct OauthClientResponse {
    client_name: String,
    client_id: String,
    redirect_uri: String,
}

#[get("/oauth_clients")]
pub async fn get_oauth_clients(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
) -> Json<AccountsResponse<OauthClientsResponse>> {
    Json(
        match oauth_client_service::get_oauth_clients(db_pool).await {
            Ok(clients) => AccountsResponse::success(OauthClientsResponse {
                oauth_clients: clients
                    .into_iter()
                    .map(|client| OauthClientResponse {
                        client_name: client.client_name,
                        client_id: client.client_id,
                        redirect_uri: client.redirect_uri,
                    })
                    .collect(),
            }),
            Err(err) => {
                error!("Failed to get oauth_clients, err {}", err);
                AccountsResponse::error(ApiError::InternalError)
            }
        },
    )
}

#[derive(Serialize, Deserialize)]
pub struct NewClientRequest {
    pub client_name: String,
    pub redirect_uri: String,
}

#[derive(Serialize)]
pub struct NewClientResponse {
    pub client_id: String,
    pub client_secret: String,
}

#[post("/oauth_clients", data = "<request>")]
pub async fn post_new_client(
    db_pool: &State<sqlx::Pool<DB>>,
    request: Json<NewClientRequest>,
    _admin_session: AdminSession,
) -> Json<AccountsResponse<NewClientResponse>> {
    Json(
        match oauth_client_service::create_oauth_client(
            db_pool,
            request.client_name.to_owned(),
            request.redirect_uri.to_owned(),
        )
        .await
        {
            Ok(client) => AccountsResponse::success(NewClientResponse {
                client_id: client.client_id,
                client_secret: client.client_secret,
            }),
            Err(OauthClientError::ClientNameTaken) => {
                error!("The client name has already been taken");
                AccountsResponse::error(ApiError::OauthClientNameTaken)
            }
            Err(err) => {
                error!("Failed to create oauth client, err: {}", err);
                AccountsResponse::error(ApiError::InternalError)
            }
        },
    )
}
