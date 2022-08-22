use rocket::{response::Redirect, State};
use sqlx::Pool;

use crate::{api::response::ResponseStatus, db::DB, services::session_service::Session};

/// First step in the oauth2 authorization flow.
#[get("/authorize?<response_type>&<client_id>&<redirect_uri>&<state>")]
pub fn get_authorization(
    db_pool: &State<Pool<DB>>,
    response_type: String,
    client_id: String,
    redirect_uri: String,
    state: String,
    _session: Session,
) -> Result<Redirect, ResponseStatus<()>> {
    Ok(Redirect::found(redirect_uri))
}
