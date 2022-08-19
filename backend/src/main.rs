#![forbid(unsafe_code)]
#![feature(try_trait_v2)]
#[macro_use]
extern crate rocket;

use api::response::{AccountsResponse, ApiError};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{fs::FileServer, Request};
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

use mobc::Pool;
use mobc_redis::redis;
use mobc_redis::RedisConnectionManager;

const MAX_REDIS_CONNECTONS: u64 = 20;

#[launch]
async fn rocket() -> _ {
    // Load
    let config = Config::new().expect("Failed to load config");

    // Setup DB
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to DB");
    db::init(&db_pool).await.expect("Failed to initialize db");

    // Setup Redis cache
    let redis_client = redis::Client::open(config.redis_url.clone()).expect(&format!(
        "Failed to connect to redis on URL {}",
        config.redis_url
    ));
    let redis_manager = RedisConnectionManager::new(redis_client);
    let redis_pool = Pool::builder()
        .max_open(MAX_REDIS_CONNECTONS)
        .build(redis_manager);

    // Test redis connection
    redis_pool
        .get()
        .await
        .expect("Test connection to redis pool failed");

    // Setup background tasks
    let pool_clone = db_pool.clone();
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
                api::logout::post_logout,
                api::me::get_me,
                api::users::get_users,
                api::whitelist::get_whitelist,
                api::whitelist::add_email_to_whitelist,
                api::whitelist::delete_email_from_whitelist,
                api::oauth_client::get_oauth_clients,
                api::oauth_client::post_new_client,
                api::oauth_client::delete_client,
                api::oauth::authorize::get_authorization,
            ],
        )
        .mount("/api/admin", routes![api::admin_view::get_admin_view,])
        .mount("/api/public", FileServer::from("static/public"))
        .register("/", catchers![unauthorized, forbidden])
        .manage(db_pool.clone())
        .manage(redis_pool)
        .manage(config)
        .attach(Template::fairing())
}

struct UnauthorizedResponse(Json<AccountsResponse<()>>);

impl<'r> Responder<'r, 'r> for UnauthorizedResponse {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'r> {
        rocket::Response::build_from(self.0.respond_to(req)?)
            .raw_header("location", "/api/login")
            .ok()
    }
}

#[catch(401)]
fn unauthorized() -> UnauthorizedResponse {
    UnauthorizedResponse(Json(AccountsResponse::error(ApiError::Unauthorized)))
}

const FORBIDDEN_TEMPLATE_NAME: &str = "forbidden-handler";

#[catch(403)]
fn forbidden(_req: &Request) -> Template {
    Template::render(FORBIDDEN_TEMPLATE_NAME, ())
}
