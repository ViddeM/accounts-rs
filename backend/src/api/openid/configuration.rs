use rocket::serde::json::Json;

use crate::util::config::{self, Config};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIdConfiguration {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    jwks_uri: String,
    scopes_supported: Vec<String>,
    response_types_supported: Vec<String>,
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
        authorization_endpoint: format!("{}/api/oauth/authorize"),
        token_endpoint: format!("{}/api/oauth/token"),
        userinfo_endpoint: format!("{}/api/openid/userinfo"),
        jwks_uri: default(),      // TODO: Not implemented
        scopes_supported: vec![], // TODO: We should probably support scopes, at least openid (as it is required by the spec).
        response_types_supported: vec!["code".to_string()],
        grant_types_supported: vec!["authorization_code".to_string()],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec![], // TODO: Not implemented
        claims_supported: vec!["email".to_string()],   // TODO: Support aud, exp, iss, iat, sub etc.
    }))
}
