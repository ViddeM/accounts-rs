use std::sync::Arc;

use crate::services::google_api_service;
use crate::services::google_api_service::GoogleApiError;
use crate::util::config::{EmailConfig, GoogleEmailConfig};

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Google API error")]
    GoogleApiError(#[from] GoogleApiError),
}

pub enum EmailProvider {
    None,
    Stdout,
    Google(Arc<GoogleEmailConfig>),
}

impl From<&EmailConfig> for EmailProvider {
    fn from(config: &EmailConfig) -> Self {
        match config {
            EmailConfig::None => EmailProvider::None,
            EmailConfig::Stdout => EmailProvider::Stdout,
            EmailConfig::Google(config) => EmailProvider::Google(config.clone()),
        }
    }
}

impl EmailProvider {
    pub async fn send_email(
        &self,
        receiver_email: &str,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailError> {
        match self {
            EmailProvider::None => Ok(()),
            EmailProvider::Stdout => stdout_send_email(receiver_email, subject, content).await,
            EmailProvider::Google(config) => {
                google_send_email(config, receiver_email, subject, content).await
            }
        }
    }
}

async fn stdout_send_email(
    receiver_email: &str,
    subject: &str,
    content: &str,
) -> Result<(), EmailError> {
    log::info!("stdout set as email provider, printing email now.");
    println!(
        "## EMAIL ##
To: {}
Subject: {}

{}      
        ",
        receiver_email, subject, content
    );

    Ok(())
}

pub async fn google_send_email(
    config: &GoogleEmailConfig,
    receiver_email: &str,
    subject: &str,
    content: &str,
) -> Result<(), EmailError> {
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

    Ok(())
}
