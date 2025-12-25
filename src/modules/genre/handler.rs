use super::dto::{CreateGenreRequest, GenreResponse, UpdateGenreRequest};
use super::service::GenreService;
use crate::common::response::{ApiError, ApiResponse, ApiSuccess};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

/// List all genres
#[utoipa::path(
    get,
    path = "/api/v1/genres",
    responses(
        (status = 200, description = "List of genres", body = ApiResponse<Vec<GenreResponse>>)
    ),
    tag = "Content"
)]
pub async fn list_genres(State(state): State<AppState>) -> impl IntoResponse {
    match GenreService::find_all(state).await {
        Ok(genres) => ApiSuccess(
            ApiResponse::success(genres, "Genres retrieved successfully"),
            StatusCode::OK,
        )
        .into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

/// Create a new genre
#[utoipa::path(
    post,
    path = "/api/v1/genres",
    request_body = CreateGenreRequest,
    responses(
        (status = 201, description = "Genre created", body = ApiResponse<GenreResponse>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn create_genre(
    State(state): State<AppState>,
    Json(payload): Json<CreateGenreRequest>,
) -> impl IntoResponse {

    match GenreService::create(state, payload).await {
        Ok(genre) => ApiSuccess(
            ApiResponse::success(genre, "Genre created successfully"),
            StatusCode::CREATED,
        )
        .into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::BAD_REQUEST).into_response(),
    }
}

/// Get genre by ID
#[utoipa::path(
    get,
    path = "/api/v1/genres/{id}",
    params(
        ("id" = Uuid, Path, description = "Genre ID")
    ),
    responses(
        (status = 200, description = "Genre details", body = ApiResponse<GenreResponse>),
        (status = 404, description = "Genre not found")
    ),
    tag = "Content"
)]
pub async fn get_genre(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match GenreService::find_by_id(state, id).await {
        Ok(genre) => ApiSuccess(
            ApiResponse::success(genre, "Genre retrieved successfully"),
            StatusCode::OK,
        )
        .into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::NOT_FOUND).into_response(),
    }
}

/// Update genre
#[utoipa::path(
    put,
    path = "/api/v1/genres/{id}",
    params(
        ("id" = Uuid, Path, description = "Genre ID")
    ),
    request_body = UpdateGenreRequest,
    responses(
        (status = 200, description = "Genre updated", body = ApiResponse<GenreResponse>),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Genre not found"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn update_genre(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateGenreRequest>,
) -> impl IntoResponse {

    match GenreService::update(state, id, payload).await {
        Ok(genre) => ApiSuccess(
            ApiResponse::success(genre, "Genre updated successfully"),
            StatusCode::OK,
        )
        .into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::BAD_REQUEST).into_response(),
    }
}

/// Delete genre
#[utoipa::path(
    delete,
    path = "/api/v1/genres/{id}",
    params(
        ("id" = Uuid, Path, description = "Genre ID")
    ),
    responses(
        (status = 200, description = "Genre deleted", body = ApiResponse<String>),
        (status = 404, description = "Genre not found"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn delete_genre(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {

    match GenreService::delete(state, id).await {
        Ok(_) => ApiSuccess(
            ApiResponse::success((), "Genre deleted successfully"),
            StatusCode::OK,
        )
        .into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::NOT_FOUND).into_response(),
    }
}

