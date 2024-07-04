use aes_gcm::{Aes256Gcm, Key, KeyInit};
use argon2::{Algorithm, Argon2, Params, Version};
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use serde::{Deserialize, Serialize};
use std::env::VarError;
use std::path::Path;
use std::sync::Arc;
use std::{env, fs, io};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Environment variable error")]
    EnvVarError(#[from] VarError),
    #[error("Empty variable error `{0}`")]
    VarEmpty(String),
    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("Invalid bool `{0}`")]
    InvalidBool(String),
    #[error("Invalid email provider `{0}`. Valid options are: none, stdout, google")]
    InvalidEmailProvider(String),
    #[error("Failed to parse signing key")]
    OpensslError(#[from] openssl::error::ErrorStack),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

// argon2 configs, determined from https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id
// Slightly more than 37 MiB
const MINIMUM_MEMORY_SIZE: u32 = 38798;
const MINIMUM_ITERATIONS: u32 = 1;
const DEGREE_OF_PARALLELISM: u32 = 1;
const REQUIRED_PEPPER_BYTES: usize = 32;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub pepper_cipher: Aes256Gcm,
    pub argon2: Argon2<'static>,
    pub email: EmailConfig,
    pub backend_address: String,
    pub redis_url: String,
    pub log_db_statements: bool,
    pub jwt_signing_key: Rsa<Private>,
}

#[derive(Clone)]
pub enum EmailConfig {
    /// No emails are sent.
    None,

    /// Emails that would be sent are instead logged.
    Stdout,

    /// Emails are sent using Google as an email provider.
    Google(Arc<GoogleEmailConfig>),
}

#[derive(Clone)]
pub struct GoogleEmailConfig {
    pub send_from_email_address: String,
    pub service_account: GoogleServiceAccount,
}

impl Config {
    pub fn new() -> ConfigResult<Config> {
        dotenvy::dotenv().ok();

        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::default(),
            Params::new(
                MINIMUM_MEMORY_SIZE,
                MINIMUM_ITERATIONS,
                DEGREE_OF_PARALLELISM,
                None,
            )
            .expect("Failed to setup argon2 parameters"),
        );

        let pepper = load_env_str("PEPPER")?;
        if pepper.len() != REQUIRED_PEPPER_BYTES {
            panic!("Pepper must be exactly {} bytes", REQUIRED_PEPPER_BYTES);
        }
        let pepper_key: &Key<Aes256Gcm> = pepper.as_bytes().into();
        let pepper_cipher = Aes256Gcm::new(pepper_key);

        let email_provider = load_env_str("EMAIL_PROVIDER")?;
        let email = match email_provider.as_str() {
            "none" => EmailConfig::None,
            "stdout" => EmailConfig::Stdout,
            "google" => {
                let send_from_email_address = load_env_str("SEND_FROM_EMAIL_ADDRESS")?;
                let service_account_file = load_env_str("SERVICE_ACCOUNT_FILE")?;
                let file_contents = fs::read_to_string(service_account_file)?;
                let service_account: GoogleServiceAccount = serde_json::from_str(&file_contents)?;
                EmailConfig::Google(Arc::new(GoogleEmailConfig {
                    send_from_email_address,
                    service_account,
                }))
            }
            _ => return Err(ConfigError::InvalidEmailProvider(email_provider)),
        };

        // JWT signing
        let key_path = load_env_str("JWT_SIGNING_KEY_PATH")?;
        let file_contents = fs::read(Path::new(&key_path))?;
        let jwt_signing_key: Rsa<Private> = Rsa::private_key_from_pem(file_contents.as_slice())?;

        Ok(Config {
            database_url: load_env_str("DATABASE_URL")?,
            pepper_cipher,
            argon2,
            email,
            backend_address: load_env_str("BACKEND_ADDRESS")?,
            redis_url: load_env_str("REDIS_URL")?,
            log_db_statements: load_env_bool("LOG_DB_STATEMENTS")?,
            jwt_signing_key: jwt_signing_key,
        })
    }
}

fn load_env_str(key: &str) -> ConfigResult<String> {
    let var = env::var(key)?;

    if var.is_empty() {
        return Err(ConfigError::VarEmpty(key.to_string()));
    }

    Ok(var)
}

fn load_env_bool(key: &str) -> ConfigResult<bool> {
    let var = load_env_str(key)?;
    match var.as_str() {
        "false" => Ok(false),
        "true" => Ok(true),
        _ => Err(ConfigError::InvalidBool(var)),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GoogleServiceAccount {
    #[serde(rename = "type")]
    pub account_type: String,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_x509_cert_url: String,
}
