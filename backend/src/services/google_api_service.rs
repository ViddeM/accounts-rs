use crate::util::config::Config;
use chrono::{Duration, Utc};
use jwt;
use jwt::{PKeyWithDigest, SignWithKey};
use openssl;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum GoogleApiError {
    #[error("OpenSSL error stack")]
    OpenSSLError(#[from] openssl::error::ErrorStack),
    #[error("JWT error")]
    JwtError(#[from] jwt::Error),
    #[error("Reqwest error `{0}`")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Google api error")]
    GoogleApiError,
}

const GRANT_TYPE_SERVICE_ACCOUNT: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
#[derive(Serialize, Deserialize)]
struct GoogleAuthRequest {
    grant_type: String,
    assertion: String,
}

impl GoogleAuthRequest {
    fn new(assertion: String) -> Self {
        Self {
            grant_type: GRANT_TYPE_SERVICE_ACCOUNT.to_string(),
            assertion,
        }
    }
}

const GMAIL_SEND_EMAIL_SCOPE: &str = "https://www.googleapis.com/auth/gmail.send";

const GOOGLE_AUD_VALUE: &str = "https://oauth2.googleapis.com/token";

fn create_jwt(config: &Config) -> Result<String, GoogleApiError> {
    let private_key = PKey::private_key_from_pem(config.service_account.private_key.as_bytes())?;
    let key_with_digest = PKeyWithDigest {
        digest: MessageDigest::sha256(),
        key: private_key,
    };

    let mut claims: BTreeMap<&str, &str> = BTreeMap::new();

    claims.insert("iss", &config.service_account.client_email);
    claims.insert("scope", GMAIL_SEND_EMAIL_SCOPE);
    claims.insert("aud", GOOGLE_AUD_VALUE);

    let now = Utc::now();
    let now_timestamp = now.timestamp().to_string();
    claims.insert("iat", &now_timestamp);

    let exp_time = now + Duration::hours(1);
    let exp_time_timestamp = exp_time.timestamp().to_string();
    claims.insert("exp", &exp_time_timestamp);
    claims.insert("sub", &config.send_from_email_address);

    Ok(claims.sign_with_key(&key_with_digest)?)
}

#[derive(Serialize, Deserialize)]
struct GoogleAuthResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

pub async fn retrieve_token(config: &Config) -> Result<String, GoogleApiError> {
    let jwt = create_jwt(config)?;

    let client = reqwest::Client::new();

    // GoogleAuthResponse
    let response_text = client
        .post(&config.service_account.token_uri)
        .form(&GoogleAuthRequest::new(jwt))
        .send()
        .await?
        .text()
        .await?;

    let response: GoogleAuthResponse = serde_json::from_str(&response_text).map_err(|_| {
        println!(
            "GOOGLE API ERR: Failed to retrieve token, err: {}",
            response_text
        );
        GoogleApiError::GoogleApiError
    })?;

    Ok(response.access_token)
}

#[derive(Serialize, Deserialize)]
struct GoogleSendEmailRequest {
    raw: String,
}

impl GoogleSendEmailRequest {
    fn new(from: &str, to: &str, subject: &str, content: &str) -> Self {
        let raw_message = base64::encode(format!(
            "From: {}\r\nTo: {}\r\nSubject: {}\r\n\r\n{}\r\n",
            from, to, subject, content
        ));
        Self { raw: raw_message }
    }
}

// Note: the `/me/` is a parameter for the CLIENT_ID and
const SEND_EMAIL_ENDPOINT: &str = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";

#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct GoogleSendEmailResponse {
    id: String,
    thread_id: String,
    label_ids: Vec<String>,
}

pub async fn send_email(
    receiver_email: &str,
    subject: &str,
    content: &str,
    token: &str,
    config: &Config,
) -> Result<(), GoogleApiError> {
    let send_email_request = GoogleSendEmailRequest::new(
        &config.send_from_email_address,
        receiver_email,
        subject,
        content,
    );

    let client = reqwest::Client::new();
    let response_text = client
        .post(SEND_EMAIL_ENDPOINT)
        .query(&[("alt", "json"), ("prettyPrint", "false")])
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .json(&send_email_request)
        .send()
        .await?
        .text()
        .await?;

    let _response: GoogleSendEmailResponse =
        serde_json::from_str(&response_text).map_err(|_| {
            println!(
                "GOOGLE API ERR: Failed to send email, err: {}",
                response_text
            );
            GoogleApiError::GoogleApiError
        })?;

    Ok(())
}
