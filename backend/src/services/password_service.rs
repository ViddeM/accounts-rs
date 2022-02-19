use crate::util::config::Config;
use aes_gcm::aead::Aead;
use aes_gcm::{Error, Nonce};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};
use std::num::ParseIntError;
use std::str::Utf8Error;

pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_PASSWORD_LENGTH: usize = 128;

#[derive(Debug)]
pub enum PasswordErr {
    Argon2Error(argon2::Error),
    PasswordHashError(argon2::password_hash::Error),
    Utf8Error(Utf8Error),
    AesGcmError(aes_gcm::Error),
}

impl From<argon2::Error> for PasswordErr {
    fn from(err: argon2::Error) -> Self {
        PasswordErr::Argon2Error(err)
    }
}

impl From<argon2::password_hash::Error> for PasswordErr {
    fn from(err: argon2::password_hash::Error) -> Self {
        PasswordErr::PasswordHashError(err)
    }
}

impl From<Utf8Error> for PasswordErr {
    fn from(err: Utf8Error) -> Self {
        PasswordErr::Utf8Error(err)
    }
}

impl From<aes_gcm::Error> for PasswordErr {
    fn from(err: Error) -> Self {
        PasswordErr::AesGcmError(err)
    }
}

type PasswordResult<T> = Result<T, PasswordErr>;

pub fn hash_and_encrypt_password(
    password: String,
    config: &Config,
) -> PasswordResult<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = config
        .argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    let nonce_arr: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_arr);

    let peppered_password_hash = config
        .pepper_cipher
        .encrypt(nonce, password_hash.as_bytes())?;

    let hexed_password: String = to_hex(peppered_password_hash);
    let hexed_nonces: String = to_hex(nonce_arr.to_vec());

    Ok((hexed_password, hexed_nonces))
}

pub fn verify_password(
    provided_password: String,
    stored_password: String,
    stored_nonce: String,
    config: &Config,
) -> bool {
    // Convert nonces from hex format to bytes
    let nonce_bytes: Vec<u8> = match from_hex(stored_nonce) {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to convert stored nonces to bytes: {}", e);
            return false;
        }
    };
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Convert stored password from hex-string to bytes
    let encrypted_password = match from_hex(stored_password) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to convert stored password to bytes: {}", e);
            return false;
        }
    };

    // Decrypt password using the pe
    let decrypted_password = match config
        .pepper_cipher
        .decrypt(nonce, encrypted_password.as_ref())
    {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to decrypt stored password using pepper, err: {}", e);
            return false;
        }
    };

    let decrypted_password_str = match std::str::from_utf8(decrypted_password.as_slice()) {
        Ok(val) => val,
        Err(err) => {
            error!("Failed to parse decrypted password as utf8, err: {}", err);
            return false;
        }
    };

    let parsed_hash = match PasswordHash::new(decrypted_password_str) {
        Ok(val) => val,
        Err(err) => {
            error!("Failed to verify password hash, err: {}", err);
            return false;
        }
    };

    config
        .argon2
        .verify_password(provided_password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn to_hex(bytes: Vec<u8>) -> String {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

fn from_hex(hex_str: String) -> Result<Vec<u8>, ParseIntError> {
    (0..hex_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16))
        .collect()
}
