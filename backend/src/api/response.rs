use serde::Serialize;

#[derive(Serialize)]
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

pub enum ApiError {
    InternalError,
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let err_str = match self {
            ApiError::InternalError => "internal_error",
        };

        serializer.serialize_str(err_str)
    }
}
