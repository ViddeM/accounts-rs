#![feature(try_trait_v2)]
mod api;
mod db;
pub mod models;
pub mod services;
pub mod util;

use crate::db::account::AccountRepository;
use crate::services::password_service::{hash_and_encrypt_password, verify_password};
use crate::util::config::Config;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use sqlx::postgres::PgPoolOptions;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() {
    let config = Config::new().expect("Failed to load config");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to DB");

    rocket::build()
        .mount(
            "/api",
            routes![
                api::index::index,
                api::login::get_login_page,
                api::login::post_login,
                api::create_account::create_account,
                api::create_account::get_create_account,
            ],
        )
        .manage(AccountRepository::new(pool))
        .manage(config)
        .attach(Template::fairing())
        .mount("/api/public", FileServer::from("static/public"))
        .launch()
        .await
        .expect("Rocket failed to start");
}
