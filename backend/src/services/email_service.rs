use crate::services::google_api_service;
use crate::services::google_api_service::GoogleApiError;
use crate::util::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Google API error")]
    GoogleApiError(#[from] GoogleApiError),
}

pub async fn send_email(
    receiver_email: String,
    subject: String,
    content: String,
    config: &Config,
) -> Result<(), EmailError> {
    let token = google_api_service::retrieve_token(config).await?;
    google_api_service::send_email(receiver_email, subject, content, token, config).await?;
    Ok(())
}
