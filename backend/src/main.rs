#![forbid(unsafe_code)]
// `rocket` macros generate a bunch of clippy warnings.
// TODO: remove these when updating rocket past 0.5.0.
#![allow(clippy::blocks_in_conditions)]
#![allow(clippy::to_string_in_format_args)]

#[macro_use]
extern crate rocket;

use std::str::FromStr;

use api::core::core_routes;
use api::external::external_routes;
use api::frontend::site_routes;
use api::oauth::oauth_routes;
use api::response::{ErrMsg, ResponseStatus};
use rocket::http::Status;
use rocket::response::Responder;
use rocket::{fs::FileServer, Request};
use rocket_dyn_templates::Template;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;
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

const MAX_REDIS_CONNECTIONS: u64 = 20;

#[launch]
async fn rocket() -> _ {
    // Load
    let config = Config::new().expect("Failed to load config");

    // Setup DB
    let mut pg_options =
        PgConnectOptions::from_str(&config.database_url).expect("Invalid database url provided");

    if !config.log_db_statements {
        pg_options = pg_options.disable_statement_logging();
    }

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_options)
        .await
        .expect("Failed to connect to DB");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    db::init(&db_pool).await.expect("Failed to initialize db");

    // Setup Redis cache
    let redis_client = redis::Client::open(config.redis_url.clone())
        .unwrap_or_else(|_| panic!("Failed to connect to redis on URL {}", config.redis_url));
    let redis_manager = RedisConnectionManager::new(redis_client);
    let redis_pool = Pool::builder()
        .max_open(MAX_REDIS_CONNECTIONS)
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
        .mount("/api/core", core_routes())
        .mount("/api/site", site_routes())
        .mount("/api/oauth", oauth_routes())
        .mount("/api/external", external_routes())
        .mount("/api/public", FileServer::from("static/public"))
        .register("/", catchers![unauthorized, forbidden])
        .manage(db_pool.clone())
        .manage(redis_pool)
        .manage(config)
        .attach(Template::fairing())
}

struct UnauthorizedResponse(ResponseStatus<()>);

impl<'r> Responder<'r, 'r> for UnauthorizedResponse {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'r> {
        rocket::Response::build_from(self.0.respond_to(req)?)
            .raw_header("location", "/api/core/login")
            .ok()
    }
}

#[catch(401)]
fn unauthorized() -> UnauthorizedResponse {
    UnauthorizedResponse(ResponseStatus::err(
        Status::Unauthorized,
        ErrMsg::Unauthorized,
    ))
}

const FORBIDDEN_TEMPLATE_NAME: &str = "forbidden-handler";

#[catch(403)]
fn forbidden(_req: &Request) -> Template {
    Template::render(FORBIDDEN_TEMPLATE_NAME, ())
}
