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

pub struct Config {
    pub database_url: String,
    pub pepper: String,
}

impl Config {
    pub fn new() -> ConfigResult<Config> {
        dotenv::dotenv().ok();

        Ok(Config {
            database_url: "".to_string(),
            pepper: "".to_string(),
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
