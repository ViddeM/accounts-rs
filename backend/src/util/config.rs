use aes_gcm::NewAead;
use aes_gcm::{Aes256Gcm, Key};
use argon2::{Algorithm, Argon2, Params, Version};
use dotenv;
use serde::{Deserialize, Serialize};
use std::env::VarError;
use std::{env, fs, io};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Environment variable error")]
    EnvVarError(#[from] VarError),
    #[error("Empty variable error")]
    VarEmpty(String),
    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("Invalid bool `{0}`")]
    InvalidBool(String),
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
    pub service_account: ServiceAccount,
    pub send_from_email_address: String,
    pub backend_address: String,
    pub offline_mode: bool,
    pub redis_url: String,
    pub log_db_statements: bool,
}

impl Config {
    pub fn new() -> ConfigResult<Config> {
        dotenv::dotenv().ok();

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

        let pepper = load_env_str(String::from("PEPPER"))?;
        if pepper.len() != REQUIRED_PEPPER_BYTES {
            panic!("Pepper must be exactly {} bytes", REQUIRED_PEPPER_BYTES);
        }
        let pepper_key = Key::from_slice(pepper.as_bytes());
        let pepper_cipher = Aes256Gcm::new(pepper_key);

        // Load service account file
        let service_account_file = load_env_str(String::from("SERVICE_ACCOUNT_FILE"))?;
        let file_contents = fs::read_to_string(service_account_file)?;
        let service_account: ServiceAccount = serde_json::from_str(&file_contents)?;

        Ok(Config {
            database_url: load_env_str(String::from("DATABASE_URL"))?,
            pepper_cipher,
            argon2,
            service_account,
            send_from_email_address: load_env_str(String::from("SEND_FROM_EMAIL_ADDRESS"))?,
            backend_address: load_env_str(String::from("BACKEND_ADDRESS"))?,
            offline_mode: load_env_bool(String::from("OFFLINE_MODE"))?,
            redis_url: load_env_str(String::from("REDIS_URL"))?,
            log_db_statements: load_env_bool(String::from("LOG_DB_STATEMENTS"))?,
        })
    }
}

fn load_env_str(key: String) -> ConfigResult<String> {
    let var = env::var(&key)?;

    if var.is_empty() {
        return Err(ConfigError::VarEmpty(key));
    }

    Ok(var)
}

fn load_env_bool(key: String) -> ConfigResult<bool> {
    let var = load_env_str(key)?;
    match var.as_str() {
        "false" => Ok(false),
        "true" => Ok(true),
        _ => Err(ConfigError::InvalidBool(var)),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceAccount {
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
