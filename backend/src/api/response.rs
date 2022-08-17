use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountsResponse<T> {
    Success(SuccessResponse<T>),
    Error(ErrorResponse),
}

impl<T> AccountsResponse<T> {
    pub fn success(data: T) -> AccountsResponse<T> {
        return AccountsResponse::Success(SuccessResponse { data });
    }

    pub fn error(err: ApiError) -> AccountsResponse<T> {
        return AccountsResponse::Error(ErrorResponse { error: err });
    }
}

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    data: T,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: ApiError,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiError {
    InternalError,
    Unauthorized,
    OauthClientNameTaken,
}
