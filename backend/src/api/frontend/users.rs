use rocket::State;
use serde::Serialize;

use crate::{
    api::{auth::admin_session_guard::AdminSession, response::ResponseStatus},
    db::DB,
    models::authority::AuthorityLevel,
    services::users_service,
};

#[derive(Serialize, Clone)]
pub struct Users {
    users: Vec<User>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    first_name: String,
    last_name: String,
    authority: AuthorityLevel,
}

#[get("/users")]
pub async fn get_users(
    db_pool: &State<sqlx::Pool<DB>>,
    _admin_session: AdminSession,
) -> ResponseStatus<Users> {
    let accs = match users_service::get_all_users(db_pool).await {
        Ok(accs) => accs,
        Err(err) => {
            error!("Failed to get all users, err: {}", err);
            return ResponseStatus::internal_err();
        }
    };

    ResponseStatus::ok(Users {
        users: accs
            .into_iter()
            .map(|acc| User {
                first_name: acc.first_name,
                last_name: acc.last_name,
                authority: acc.authority,
            })
            .collect::<Vec<User>>(),
    })
}
