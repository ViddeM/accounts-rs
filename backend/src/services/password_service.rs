use crate::util::accounts_error::AccountsResult;
use crate::util::config::Config;
use aead::generic_array::GenericArray;
use aes::{BlockDecrypt, BlockEncrypt};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, Error, PasswordHash, PasswordHasher, PasswordVerifier};
use std::str::Utf8Error;

enum PasswordErr {
    Argon2Error(argon2::Error),
    PasswordHashError(argon2::password_hash::Error),
    Utf8Error(Utf8Error),
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

type PasswordResult<T> = Result<T, PasswordErr>;

pub fn hash_and_encrypt_password(password: String, config: Config) -> PasswordResult<String> {
    let salt = SaltString::generate(&mut OsRng);

    let mut password_hash = config
        .argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    let mut password_arr = GenericArray::from_slice(password_hash.as_bytes());
    config.pepper_cipher.encrypt_block(&mut password_arr);

    Ok(std::str::from_utf8(password_arr.as_slice())?.to_string())
}

pub fn verify_password(provided_password: String, stored_password: String, config: Config) -> bool {
    let mut decrypted_password = GenericArray::from_slice(stored_password.as_bytes());
    config.pepper_cipher.decrypt_block(&mut decrypted_password);

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
