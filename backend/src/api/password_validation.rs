use std::collections::BTreeMap;

use crate::services::password_service;

const MIN_PASSWORD_LEN_KEY: &str = "min_password_len";
const MAX_PASSWORD_LEN_KEY: &str = "max_password_len";

const ERR_PASSWORDS_NOT_MATCH: &str = "Passwords does not match";
const ERR_PASSWORD_TOO_SHORT: &str = "The password is too short";
const ERR_PASSWORD_TOO_LONG: &str = "The password is too long";

pub fn get_default_password_page_data() -> BTreeMap<&'static str, String> {
    let mut data: BTreeMap<&str, String> = BTreeMap::new();
    let min_password_length = password_service::MIN_PASSWORD_LENGTH.to_string();
    let max_password_length = password_service::MAX_PASSWORD_LENGTH.to_string();
    data.insert(MIN_PASSWORD_LEN_KEY, min_password_length);
    data.insert(MAX_PASSWORD_LEN_KEY, max_password_length);
    data
}

pub fn validate_new_passwords(password: &str, password_repeat: &str) -> Result<(), &'static str> {
    if password != password_repeat {
        return Err(ERR_PASSWORDS_NOT_MATCH);
    }

    if password.len() < password_service::MIN_PASSWORD_LENGTH {
        return Err(ERR_PASSWORD_TOO_SHORT);
    }

    if password.len() > password_service::MAX_PASSWORD_LENGTH {
        return Err(ERR_PASSWORD_TOO_LONG);
    }

    Ok(())
}
