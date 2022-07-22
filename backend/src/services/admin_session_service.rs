use rocket::{http::Status, request::FromRequest, State};

use crate::{
    db::{account_repository, new_transaction, DB},
    models::account::Account,
    services::session_service::Session,
};

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

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let session = match request.guard::<Session>().await.succeeded() {
            Some(s) => s,
            None => {
                error!("Failed to retrieve session");
                return rocket::request::Outcome::Failure((
                    Status::Unauthorized,
                    AdminSessionError::MissingSession,
                ));
            }
        };

        let db_pool = match request.guard::<&State<sqlx::Pool<DB>>>().await.succeeded() {
            Some(db) => db,
            None => {
                error!("Failed to retrieve db pool!");
                return rocket::request::Outcome::Failure((
                    Status::InternalServerError,
                    AdminSessionError::DBError,
                ));
            }
        };

        let mut transaction = match new_transaction(db_pool).await {
            Ok(t) => t,
            Err(err) => {
                error!("Failed to create new transaction, err: {}", err);
                return rocket::request::Outcome::Failure((
                    Status::InternalServerError,
                    AdminSessionError::DBError,
                ));
            }
        };

        let admin_account =
            match account_repository::get_admin_account(&mut transaction, session.account_id).await
            {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    error!(
                        "Account {} does not have admin clearance",
                        session.account_id
                    );
                    return rocket::request::Outcome::Failure((
                        Status::Forbidden,
                        AdminSessionError::UserNotAdmin,
                    ));
                }
                Err(err) => {
                    error!("Failed to get account from DB, err: {}", err);
                    return rocket::request::Outcome::Failure((
                        Status::InternalServerError,
                        AdminSessionError::DBError,
                    ));
                }
            };

        rocket::request::Outcome::Success(AdminSession {
            session,
            account: admin_account,
        })
    }
}
