pub mod accounts_error;
mod api;
mod db;
pub mod models;

use crate::db::account::AccountRepository;
use dotenv::dotenv;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
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
        .attach(Template::fairing())
        .mount("/api/public", FileServer::from("static"))
        .launch()
        .await
        .expect("Rocket failed to start");
}
