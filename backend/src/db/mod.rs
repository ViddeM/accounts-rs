use sqlx::Postgres;

pub mod account;
pub mod login_details;
pub mod login_provider;
pub mod third_party_login;
pub mod whitelist;

pub type DB = Postgres;
