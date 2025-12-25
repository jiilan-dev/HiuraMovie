use super::dto::{LoginRequest, RegisterRequest, TokenClaims, AuthResponse, UserResponse};
use super::service::AuthService;
use crate::state::AppState;
use crate::common::response::{ApiResponse, ApiSuccess, ApiError};
use axum::{
    extract::{State, Extension},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use tower_cookies::{Cookie, Cookies};

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created successfully", body = ApiResponse<UserResponse>),
        (status = 400, description = "Bad Request")
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match AuthService::register(state, payload).await {
        Ok(user) => ApiSuccess(ApiResponse::success(user, "User registered successfully"), StatusCode::CREATED).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::BAD_REQUEST).into_response(),
    }
}

/// Login user and get tokens
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<AuthResponse>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match AuthService::login(state, payload).await {
        Ok((response, refresh_token)) => {
            let mut cookie = Cookie::new("refresh_token", refresh_token);
            cookie.set_http_only(true);
            cookie.set_path("/api/v1/auth"); // Allow access for refresh AND logout
             cookie.set_secure(false); // Keep false for HTTP localhost
            // Expiry 7 days
             cookie.set_max_age(Some(time::Duration::days(7)));

            cookies.add(cookie);
            
            ApiSuccess(ApiResponse::success(response, "Login successful"), StatusCode::OK).into_response()
        }
        Err(e) => ApiError(e.to_string(), StatusCode::UNAUTHORIZED).into_response(),
    }
}

/// Logout user
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        // Using String instead of () to avoid TypeTree panic
        (status = 200, description = "Logged out successfully", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
    Extension(claims): Extension<TokenClaims>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 1. Block Access Token
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str[7..].to_owned();
                // Block for remaining time, or just 15 mins (900s) for simplicity
                let ttl = claims.exp.saturating_sub(jsonwebtoken::get_current_timestamp() as usize);
                let _ = AuthService::block_token(state.clone(), token, ttl).await;
            }
        }
    }

    // 2. Revoke Refresh Token
    let _ = AuthService::logout(state, claims.sub).await;

    // 3. Clear Cookie
    let mut cookie = Cookie::new("refresh_token", "");
    cookie.set_path("/api/v1/auth");
    cookies.remove(cookie);
    
    ApiSuccess(ApiResponse::success((), "Logged out successfully"), StatusCode::OK).into_response()
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<AuthResponse>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Auth"
)]
pub async fn refresh(
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    let refresh_token_cookie = cookies.get("refresh_token");
    
    let refresh_token = match refresh_token_cookie {
        Some(c) => c.value().to_string(),
        None => return ApiError("Missing refresh token".to_string(), StatusCode::UNAUTHORIZED).into_response(),
    };

    tracing::info!("Refresh request received with token: {}", refresh_token); // Log the token!
    
    // Parse user_id from token "user_id:uuid"
    let parts: Vec<&str> = refresh_token.split(':').collect();
    if parts.len() != 2 {
        tracing::error!("Invalid token format: {}", refresh_token);
        return ApiError("Invalid token format".to_string(), StatusCode::UNAUTHORIZED).into_response();
    }
    
    let user_id = match uuid::Uuid::parse_str(parts[0]) {
        Ok(id) => id,
        Err(_) => return ApiError("Invalid user ID in token".to_string(), StatusCode::UNAUTHORIZED).into_response(),
    };

    match AuthService::refresh_access(state, refresh_token, user_id).await {
        Ok((response, new_refresh_token)) => {
             let mut cookie = Cookie::new("refresh_token", new_refresh_token);
            cookie.set_http_only(true);
            cookie.set_path("/api/v1/auth"); // Allow access for refresh AND logout
             cookie.set_secure(false); // Keep false for HTTP localhost
            // Expiry 7 days
             cookie.set_max_age(Some(time::Duration::days(7)));

            cookies.add(cookie);

            ApiSuccess(ApiResponse::success(response, "Token refreshed"), StatusCode::OK).into_response()
        },
        Err(e) => ApiError(e.to_string(), StatusCode::UNAUTHORIZED).into_response(),
    }
}
