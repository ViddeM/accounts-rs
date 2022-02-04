#![feature(try_trait_v2)]
#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use sqlx::postgres::PgPoolOptions;

use crate::db::account::AccountRepository;
use crate::db::login_details::LoginDetailsRepository;
use crate::db::login_provider::LoginProviderRepository;
use crate::db::third_party_login::ThirdPartyLoginRepository;
use crate::db::whitelist::WhitelistRepository;

use crate::util::config::Config;

mod api;
mod db;
pub mod models;
pub mod services;
pub mod util;

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
        .mount("/api/public", FileServer::from("static/public"))
        .manage(AccountRepository::new(pool.clone()))
        .manage(LoginDetailsRepository::new(pool.clone()))
        .manage(LoginProviderRepository::new(pool.clone()))
        .manage(ThirdPartyLoginRepository::new(pool.clone()))
        .manage(WhitelistRepository::new(pool.clone()))
        .manage(config)
        .attach(Template::fairing())
        .launch()
        .await
        .expect("Rocket failed to start");
}
