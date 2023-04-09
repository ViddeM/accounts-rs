use rocket::{http::Status, request::FromRequest, request::Outcome, State};

use crate::{
    db::{account_repository, new_transaction, DB},
    models::account::Account,
    util::uuid::uuid_to_sqlx,
};

use super::session_guard::Session;

#[derive(Debug)]
pub struct AdminSession {
    pub session: Session,
    pub account: Account,
}

#[derive(Debug, thiserror::Error)]
pub enum AdminSessionError {
    #[error("Failed to retrieve a valid session for the user")]
    MissingSession,
    #[error("Communication with the database failed")]
    DBError,
    #[error("The currently logged in user does not have admin clearance")]
    UserNotAdmin,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminSession {
    type Error = AdminSessionError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let session = match request.guard::<Session>().await {
            Outcome::Success(s) => s,
            Outcome::Failure((status, error)) => {
                error!(
                    "Failed to retrieve session, status: {}, err: {}",
                    status, error
                );
                return Outcome::Failure((Status::Unauthorized, AdminSessionError::MissingSession));
            }
            Outcome::Forward(_) => {
                error!("Failed to retrieve session, got forward response");
                return Outcome::Failure((Status::Unauthorized, AdminSessionError::MissingSession));
            }
        };

        let db_pool = match request.guard::<&State<sqlx::Pool<DB>>>().await.succeeded() {
            Some(db) => db,
            None => {
                error!("Failed to retrieve db pool!");
                return Outcome::Failure((Status::InternalServerError, AdminSessionError::DBError));
            }
        };

        let mut transaction = match new_transaction(db_pool).await {
            Ok(t) => t,
            Err(err) => {
                error!("Failed to create new transaction, err: {}", err);
                return Outcome::Failure((Status::InternalServerError, AdminSessionError::DBError));
            }
        };

        let admin_account = match account_repository::get_admin_account(
            &mut transaction,
            uuid_to_sqlx(session.account_id),
        )
        .await
        {
            Ok(Some(acc)) => acc,
            Ok(None) => {
                error!(
                    "Account {} does not have admin clearance",
                    session.account_id
                );
                return Outcome::Failure((Status::Forbidden, AdminSessionError::UserNotAdmin));
            }
            Err(err) => {
                error!("Failed to get account from DB, err: {}", err);
                return Outcome::Failure((Status::InternalServerError, AdminSessionError::DBError));
            }
        };

        Outcome::Success(AdminSession {
            session,
            account: admin_account,
        })
    }
}
