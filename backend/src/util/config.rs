use aes_gcm::NewAead;
use aes_gcm::{Aes256Gcm, Key};
use argon2::{Algorithm, Argon2, Params, Version};
use dotenv;
use std::env;
use std::env::VarError;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Environment variable error")]
    EnvVarError(#[from] VarError),
    #[error("Empty variable error")]
    VarEmpty(String),
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

        let pepper = load_env_str("PEPPER".to_string())?;
        if pepper.len() != REQUIRED_PEPPER_BYTES {
            panic!("Pepper must be exactly {} bytes", REQUIRED_PEPPER_BYTES);
        }
        let pepper_key = Key::from_slice(pepper.as_bytes());
        let pepper_cipher = Aes256Gcm::new(pepper_key);

        Ok(Config {
            database_url: load_env_str("DATABASE_URL".to_string())?,
            pepper_cipher,
            argon2,
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
