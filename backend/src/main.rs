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
use api::openid::{self, openid_routes};
use api::response::{ErrMsg, ResponseStatus};
use eyre::{eyre, WrapErr};
use rocket::http::Status;
use rocket::response::Responder;
use rocket::{fs::FileServer, Request};
use rocket_dyn_templates::Template;
use services::email_service::EmailProvider;
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

#[rocket::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // Load
    let config = Config::new().wrap_err("Failed to load config")?;

    // Setup DB
    let mut pg_options = PgConnectOptions::from_str(&config.database_url).wrap_err(eyre!(
        "Invalid database url provided {:?}",
        config.database_url
    ))?;

    if !config.log_db_statements {
        pg_options = pg_options.disable_statement_logging();
    }

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_options)
        .await
        .wrap_err("Failed to connect to DB")?;

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .wrap_err("Failed to run migrations")?;

    db::init(&db_pool)
        .await
        .wrap_err("Failed to initialize db")?;

    // Setup Redis cache
    let redis_client = redis::Client::open(config.redis_url.clone()).wrap_err(eyre!(
        "Failed to connect to redis on URL {:?}",
        config.redis_url
    ))?;
    let redis_manager = RedisConnectionManager::new(redis_client);
    let redis_pool = Pool::builder()
        .max_open(MAX_REDIS_CONNECTIONS)
        .build(redis_manager);

    // Test redis connection
    redis_pool
        .get()
        .await
        .wrap_err("Test connection to redis pool failed")?;

    // Setup background tasks
    let pool_clone = db_pool.clone();
    task::spawn(background_task::run_background_tasks(pool_clone));

    let email_provider = EmailProvider::from(&config.email);

    let rocket = rocket::build()
        .mount("/api/core", core_routes())
        .mount("/api/site", site_routes())
        .mount("/api/oauth", oauth_routes())
        .mount("/api/openid", openid_routes())
        .mount("/api/external", external_routes())
        .mount("/api/public", FileServer::from("static/public"))
        .mount(
            "/",
            routes![openid::configuration::get_openid_configuration],
        )
        .register("/", catchers![unauthorized, forbidden])
        .manage(db_pool.clone())
        .manage(redis_pool)
        .manage(config)
        .manage(email_provider)
        .attach(Template::fairing());

    rocket.launch().await?;

    Ok(())
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
