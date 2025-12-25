use crate::modules::auth::dto::TokenClaims;
use crate::state::AppState;
use crate::common::response::ApiError;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use redis::AsyncCommands;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // 1. Extract token from header
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });

    let token = match token {
        Some(t) => t,
        None => return Err(ApiError("Unauthorized: Missing or invalid token".to_string(), StatusCode::UNAUTHORIZED)),
    };

    // 2. Check if token is blocked in Redis
    let mut redis = state
        .redis
        .get_conn()
        .await
        .map_err(|_| ApiError("Internal Server Error: Redis unavailable".to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    let is_blocked: bool = redis
        .exists(format!("blocked_token:{}", token))
        .await
        .map_err(|_| ApiError("Internal Server Error: Redis error".to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    if is_blocked {
        return Err(ApiError("Unauthorized: Token is blocked/revoked".to_string(), StatusCode::UNAUTHORIZED));
    }

    // 3. Verify JWT
    // Use secret from config
    let secret = &state.config.jwt_secret;
    
    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ApiError("Unauthorized: Invalid token signature".to_string(), StatusCode::UNAUTHORIZED))?
    .claims;

    // 4. Inject claims into request extensions
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
