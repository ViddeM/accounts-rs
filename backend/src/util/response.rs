use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::serde::json::{json, Json};
use rocket::serde::Serialize;
use rocket::Request;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct ResponseStatus<T: Serialize + Clone> {
    pub status: Status,
    pub response_data: ResponseData<T>,
}

impl<T: Serialize + Clone> ResponseStatus<T> {
    pub fn ok(data: T) -> ResponseStatus<T> {
        ResponseStatus {
            status: Status::Ok,
            response_data: ResponseData::Success(data),
        }
    }

    pub fn err(status: Status, msg: ErrMsg) -> ResponseStatus<T> {
        ResponseStatus {
            status: status,
            response_data: ResponseData::Failure(msg),
        }
    }

    pub fn internal_err() -> ResponseStatus<T> {
        ResponseStatus {
            status: Status::InternalServerError,
            response_data: ResponseData::Failure(ErrMsg::InternalServerError),
        }
    }
}

#[derive(Clone, Serialize)]
pub enum ResponseData<T: Serialize + Clone> {
    Success(T),
    Failure(ErrMsg),
}

#[derive(Clone, Serialize)]
pub enum ErrMsg {
    InternalServerError,
    EmailAlreadyRegistered,
}

impl Display for ErrMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrMsg::InternalServerError => write!(f, "internal_server_error"),
            ErrMsg::EmailAlreadyRegistered => write!(f, "email_already_registered"),
        }
    }
}

impl<T: Serialize + Clone> Display for ResponseData<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseData::Success(_) => write!(f, "Success"),
            ResponseData::Failure(err_msg) => write!(f, "Error: {}", err_msg),
        }
    }
}

impl<'r, T: Serialize + Clone> Responder<'r, 'static> for ResponseStatus<T> {
    fn respond_to(self, request: &'r Request<'_>) -> Result<Response<'static>, Status> {
        if self.status.code >= 400 {
            warn!(
                "Status: {} :: ResponseData: {}",
                self.status.code, self.response_data
            )
        } else {
            info!("Success, status: {}", self.status)
        }

        let response_data = match self.response_data {
            ResponseData::Success(data) => json!({ "data": data }),
            ResponseData::Failure(err_msg) => json!({ "error_msg": err_msg }),
        };

        let mut response = Json(json!({
            "status": self.status.code,
            "response_data": response_data
        }))
        .respond_to(request)?;

        response.set_status(self.status);

        Ok(response)
    }
}
