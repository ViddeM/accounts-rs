use crate::accounts_error::AccountsResult;
use crate::response::ResponseData::Failure;
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::serde::json::{json, Json};
use rocket::Request;
use std::fmt::{Display, Formatter};
use std::ops::{FromResidual, Try};

#[derive(Debug, Clone)]
pub struct ResponseStatus<T> {
    pub status: Status,
    pub response_data: ResponseData<T>,
}

impl<T> From<AccountsResult<T>> for ResponseStatus<T> {
    fn from(_: AccountsResult<T>) -> Self {
        ResponseStatus {
            status: Status::InternalServerError,
            response_data: (Failure(format!("Internal Server Error"))),
        }
    }
}

impl<T> FromResidual for ResponseStatus<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {}
}

pub enum ResponseData<T> {
    Success(T),
    Failure(String),
}

impl<T> Display for ResponseData<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseData::Success(_) => write!(f, "Success"),
            ResponseData::Failure(err_msg) => write!(f, "Error: {}", err_msg),
        }
    }
}

impl<'r, T> Responder<'r, 'static> for ResponseStatus<T> {
    fn respond_to(self, request: &'r Request<'_>) -> Result<Response<'static>, Status> {
        if self.status.code >= 400 {
            warn!(
                "Status: {} :: ResponseData: {}",
                self.status.code, self.response_data
            )
        } else {
            info!("Success, status: {}", self.status)
        }

        let mut response = Json(json!({
            "status": self.status.code,
            "response_data": self.response_data,
        }))
        .respond_to(request)?;

        response.set_status(self.status);

        Ok(response)
    }
}
