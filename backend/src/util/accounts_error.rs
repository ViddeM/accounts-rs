use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{Request, Response};
use std::env::VarError;
use std::fmt::Debug;
use std::io::Cursor;

#[derive(Debug, thiserror::Error)]
pub enum AccountsError {
    #[error("Sqlx error")]
    SqlxError(#[from] sqlx::Error),
    #[error("Rocket error")]
    RocketError(#[from] rocket::Error),
    #[error("Environment variable error")]
    EnvVarError(#[from] VarError),
}

pub type AccountsResult<T> = Result<T, AccountsError>;

const SOMETHING_WENT_WRONG: &str = "Oops something went wrong";

impl<'r> Responder<'r, 'r> for AccountsError {
    fn respond_to(self, _request: &Request) -> rocket::response::Result<'r> {
        println!("Something went wrong: {:?}", self);

        Response::build()
            .sized_body(
                SOMETHING_WENT_WRONG.len(),
                Cursor::new(SOMETHING_WENT_WRONG),
            )
            .header(ContentType::Plain)
            .status(Status::InternalServerError)
            .ok()
    }
}
