#![feature(try_trait_v2)]

pub mod accounts_error;
mod api;
pub mod config;
mod db;
pub mod models;
pub mod response;

use crate::config::Config;
use crate::db::account::AccountRepository;
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
                api::login::post_login
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
