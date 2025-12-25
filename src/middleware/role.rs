use crate::modules::auth::dto::TokenClaims;
use crate::common::response::ApiError;
use axum::{
    extract::{Request, Extension},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn admin_guard(
    Extension(claims): Extension<TokenClaims>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if claims.role != "ADMIN" {
        return Err(ApiError("Forbidden: Admin access required".to_string(), StatusCode::FORBIDDEN));
    }

    Ok(next.run(req).await)
}
