use std::fmt::{Display, Formatter};

use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::serde::Serialize;
use rocket::Request;
use serde_json::json;

#[derive(Clone)]
pub struct ResponseStatus<T: Serialize + Clone> {
    pub status: Status,
    pub response_data: ResponseData<T>,
}

#[derive(Clone, Serialize)]
pub struct NoContent();

impl<T: Serialize + Clone> ResponseStatus<T> {
    pub fn ok_no_content() -> ResponseStatus<NoContent> {
        ResponseStatus {
            status: Status::NoContent,
            response_data: ResponseData::Success(NoContent {}),
        }
    }

    pub fn ok(data: T) -> ResponseStatus<T> {
        ResponseStatus {
            status: Status::Ok,
            response_data: ResponseData::Success(data),
        }
    }

    pub fn err(status: Status, msg: ErrMsg) -> ResponseStatus<T> {
        ResponseStatus {
            status,
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

impl<T: Serialize + Clone> Display for ResponseData<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseData::Success(_) => write!(f, "Success"),
            ResponseData::Failure(err_msg) => write!(f, "Error: {}", err_msg),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrMsg {
    InternalServerError,
    Unauthorized,
    InvalidUuid,
    OauthClientNameTaken,
    NoOauthClientWithId,
    InvalidResponseType,
}

impl Display for ErrMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self))
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
            ResponseData::Success(data) => json!({ "success": data }),
            ResponseData::Failure(err_msg) => json!({ "error_msg": err_msg }),
        };

        Response::build_from(response_data.respond_to(request)?)
            .status(self.status)
            .ok()
    }
}
