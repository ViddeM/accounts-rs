use rocket::serde::json::Json;

use crate::util::config::{self, Config};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIdConfiguration {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    jwks_uri: String,
    registration_endpoint: String,
    scopes_supported: Vec<String>,
    response_types_supported: Vec<String>,
    response_modes_supported: Vec<String>,
    grant_types_supported: Vec<String>,
    subject_types_supported: Vec<String>,
    id_token_signing_alg_values_supported: Vec<String>,
    claims_supported: Vec<String>,
}

#[derive(Responder, Debug)]
pub enum OpenIdConfigurationResponse {
    Success(Json<OpenIdConfiguration>),
}

#[get("/.well-known/openid-configuration")]
pub async fn get_openid_configuration(config: &State<Config>) -> OpenIdConfigurationResponse {
    Ok(Json(OpenIdConfiguration {
        issuer: config.backend_address,
        authorization_endpoint: todo!(),
        token_endpoint: todo!(),
        userinfo_endpoint: todo!(),
        jwks_uri: todo!(),
        registration_endpoint: todo!(),
        scopes_supported: todo!(),
        response_types_supported: todo!(),
        response_modes_supported: todo!(),
        grant_types_supported: todo!(),
        subject_types_supported: todo!(),
        id_token_signing_alg_values_supported: todo!(),
        claims_supported: todo!(),
    }))
}
