use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T, message: &str) -> Self {
        Self {
            status: "success".to_string(),
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            status: "error".to_string(),
            message: message.to_string(),
            data: None,
        }
    }
}

pub struct ApiSuccess<T>(pub T, pub StatusCode);

impl<T> IntoResponse for ApiSuccess<ApiResponse<T>>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let (response, status) = (self.0, self.1);
        (status, Json(response)).into_response()
    }
}

pub struct ApiError(pub String, pub StatusCode);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (message, status) = (self.0, self.1);
        let response = ApiResponse::<()>::error(&message);
        (status, Json(response)).into_response()
    }
}
