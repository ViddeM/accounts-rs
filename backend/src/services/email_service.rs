use crate::services::google_api_service;
use crate::services::google_api_service::GoogleApiError;
use crate::util::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Google API error")]
    GoogleApiError(#[from] GoogleApiError),
}

pub async fn send_email(
    receiver_email: &str,
    subject: &str,
    content: &str,
    config: &Config,
) -> Result<(), EmailError> {
    if config.offline_mode {
        println!(
            "OFFLINE_MODE SENDING EMAIL:
To: {}
Subject: {}

{}
        
        ",
            receiver_email, subject, content
        )
    } else {
        let token = google_api_service::retrieve_token(config)
            .await
            .map_err(|err| {
                error!("Failed to retrieve token from google, err: {}", err);
                err
            })?;
        google_api_service::send_email(receiver_email, subject, content, &token, config)
            .await
            .map_err(|err| {
                error!("Failed to send email, err: {}", err);
                err
            })?;
    }
    Ok(())
}
