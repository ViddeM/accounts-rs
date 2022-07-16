#![forbid(unsafe_code)]
#![feature(try_trait_v2)]
#[macro_use]
extern crate rocket;

use rocket::{fs::FileServer, response::Redirect, Request};
use rocket_dyn_templates::Template;
use sqlx::postgres::PgPoolOptions;
use tokio::task;

use crate::util::config::Config;

mod api;
mod background_task;
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

    db::init(&pool).await.expect("Failed to initialize db");

    let pool_clone = pool.clone();

    task::spawn(background_task::run_background_tasks(pool_clone));

    rocket::build()
        .mount(
            "/api",
            routes![
                api::index::index,
                api::login::get_login_page,
                api::login::post_login,
                api::create_account::create_account,
                api::create_account::get_create_account,
                api::login_successful::get_login_successful,
                api::activate_account::get_activate_account,
                api::password_reset::get_forgot_password,
                api::password_reset::post_forgot_password,
                api::password_reset::get_reset_password,
                api::password_reset::post_reset_password,
            ],
        )
        .mount("/api/public", FileServer::from("static/public"))
        .register("/", catchers![unauthorized])
        .manage(pool.clone())
        .manage(config)
        .attach(Template::fairing())
        .launch()
        .await
        .expect("Rocket failed to start");
}

#[catch(401)]
fn unauthorized(_req: &Request) -> Redirect {
    Redirect::found("/api/login")
}
