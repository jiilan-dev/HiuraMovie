use crate::common::response::{ApiError, ApiResponse, ApiSuccess};
use crate::common::upload::stream_to_s3;
use crate::state::AppState;
use crate::modules::content::dto::*;
use crate::modules::content::service::ContentService;
use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;
use uuid::Uuid;

// --- MOVIE HANDLERS ---

#[utoipa::path(
    post,
    path = "/api/v1/movies",
    request_body = CreateMovieRequest,
    responses(
        (status = 201, description = "Movie Created", body = ApiResponse<MovieResponse>),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn create_movie(
    State(state): State<AppState>,
    Json(req): Json<CreateMovieRequest>,
) -> impl IntoResponse {
    match ContentService::create_movie(state, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Movie created successfully").into(), StatusCode::CREATED).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/movies",
    responses(
        (status = 200, description = "List Movies", body = ApiResponse<Vec<MovieResponse>>),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content"
)]
pub async fn list_movies(State(state): State<AppState>) -> impl IntoResponse {
    match ContentService::list_movies(state).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Movies retrieved successfully").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/movies/{id}",
    params(
        ("id" = Uuid, Path, description = "Movie ID")
    ),
    responses(
        (status = 200, description = "Get Movie", body = ApiResponse<MovieResponse>),
        (status = 404, description = "Movie Not Found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content"
)]
pub async fn get_movie(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ContentService::get_movie(state, id).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Movie retrieved successfully").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

// --- SERIES HANDLERS ---

#[utoipa::path(
    post,
    path = "/api/v1/series",
    request_body = CreateSeriesRequest,
    responses(
        (status = 201, description = "Series Created", body = ApiResponse<SeriesResponse>),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn create_series(
    State(state): State<AppState>,
    Json(req): Json<CreateSeriesRequest>,
) -> impl IntoResponse {
    match ContentService::create_series(state, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Series created successfully").into(), StatusCode::CREATED).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/series",
    responses(
        (status = 200, description = "List Series", body = ApiResponse<Vec<SeriesListResponse>>),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content"
)]
pub async fn list_series(State(state): State<AppState>) -> impl IntoResponse {
    match ContentService::list_series(state).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Series retrieved successfully").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/series/{id}",
    params(
        ("id" = Uuid, Path, description = "Series ID")
    ),
    responses(
        (status = 200, description = "Get Series", body = ApiResponse<SeriesResponse>),
        (status = 404, description = "Series Not Found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content"
)]
pub async fn get_series(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ContentService::get_series(state, id).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Series retrieved successfully").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

// --- SEASON & EPISODE HANDLERS ---

#[utoipa::path(
    post,
    path = "/api/v1/seasons",
    request_body = CreateSeasonRequest,
    responses(
        (status = 201, description = "Season Created", body = ApiResponse<SeasonResponse>),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn create_season(
    State(state): State<AppState>,
    Json(req): Json<CreateSeasonRequest>,
) -> impl IntoResponse {
    match ContentService::create_season(state, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Season created successfully").into(), StatusCode::CREATED).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/episodes",
    request_body = CreateEpisodeRequest,
    responses(
        (status = 201, description = "Episode Created", body = ApiResponse<super::model::Episode>),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn create_episode(
    State(state): State<AppState>,
    Json(req): Json<CreateEpisodeRequest>,
) -> impl IntoResponse {
    match ContentService::create_episode(state, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Episode created successfully").into(), StatusCode::CREATED).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

/// Upload Movie Video
/// This is a streaming upload directly to S3/MinIO
#[utoipa::path(
    post,
    path = "/api/v1/movies/{id}/upload",
    params(
        ("id" = Uuid, Path, description = "Movie ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"), // Use String/Binary for schema
    responses(
        (status = 200, description = "Upload successful", body = ApiResponse<String>),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Movie not found"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn upload_movie_video(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 1. Check if movie exists (Using Repository)
    use crate::modules::content::repository::ContentRepository;
    
    let exists = ContentRepository::get_movie_by_id(&state.db, id).await;

    match exists {
        Ok(Some(_)) => {},
        Ok(None) => return ApiError("Movie not found".to_string(), StatusCode::NOT_FOUND).into_response(),
        Err(e) => return ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }

    // 2. Process Multipart Stream
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "video" {
            let file_name = field.file_name().unwrap_or("video.mp4").to_string();
            info!("Starting upload for movie {}: {}", id, file_name);

            let key = format!("movies/{}/master_{}", id, file_name);
            
            // STREAMING UPLOAD
            match stream_to_s3(&state.storage, field, key.clone()).await {
                Ok(_url) => {
                    // 3. Update DB (Using Service)
                    // We store the RELATIVE KEY in the DB for portability
                    if let Err(e) = ContentService::complete_movie_upload(state.clone(), id, key).await {
                         return ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response();
                    }

                    return ApiSuccess(
                        ApiResponse::success(_url, "Video uploaded successfully"),
                        StatusCode::OK
                    ).into_response();
                },
                Err(e) => {
                    return ApiError(format!("Upload failed: {}", e), StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }
            }
        }
    }

    ApiError("No video field found in multipart request".to_string(), StatusCode::BAD_REQUEST).into_response()
}

/// Upload Movie Thumbnail
/// Multipart upload to S3/MinIO (Thumbnails bucket)
#[utoipa::path(
    post,
    path = "/api/v1/movies/{id}/upload-thumbnail",
    params(
        ("id" = Uuid, Path, description = "Movie ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"), 
    responses(
        (status = 200, description = "Upload successful", body = ApiResponse<String>),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Movie not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn upload_movie_thumbnail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 1. Check if movie exists
    use crate::modules::content::repository::ContentRepository;
    
    let exists = ContentRepository::get_movie_by_id(&state.db, id).await;
    match exists {
        Ok(Some(_)) => {},
        Ok(None) => return ApiError("Movie not found".to_string(), StatusCode::NOT_FOUND).into_response(),
        Err(e) => return ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }

    // 2. Process Multipart
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "thumbnail" {
            let file_name = field.file_name().unwrap_or("thumb.jpg").to_string();
            info!("Starting thumbnail upload for movie {}: {}", id, file_name);

            // Use thumbnails bucket and specific key path
            // e.g. movies/{id}/thumbnail.jpg (preserve extension if possible, or force jpg/png)
            let extension = std::path::Path::new(&file_name).extension().and_then(|e| e.to_str()).unwrap_or("jpg");
            let key = format!("movies/{}/thumbnail.{}", id, extension);
            
            // Switch storage bucket temporarily or use the configured thumbnails bucket
            // Since StorageService is cloned with bucket config, we need a way to target the other bucket.
            // Our StorageService struct has `bucket` field.
            // We need to construct a new StorageService or modify it? 
            // Better: helper method `put_object` that accepts bucket name?
            // Current `stream_to_s3` uses `state.storage`.
            // Let's modify `stream_to_s3` or create `stream_to_s3_bucket`.
            
            // Wait, strict types in `upload.rs`.
            // Let's look at `upload.rs`.
            // For now, let's assume we can clone storage and set bucket? No, bucket is public String but `client` is shared.
            
            let mut storage_for_thumb = state.storage.clone();
            storage_for_thumb.bucket = state.config.minio_bucket_thumbnails.clone();

            match stream_to_s3(&storage_for_thumb, field, key.clone()).await {
                Ok(_url) => {
                    // 3. Update DB
                    // Store relative key but maybe prefixed with bucket? 
                    // Or usually we allow frontend to guess or backend to serve it via proxy.
                    // For now, save relative key.
                    if let Err(e) = ContentService::complete_movie_thumbnail_upload(state.clone(), id, key).await {
                         return ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response();
                    }

                    return ApiSuccess(
                        ApiResponse::success(_url, "Thumbnail uploaded successfully"),
                        StatusCode::OK
                    ).into_response();
                },
                Err(e) => {
                    return ApiError(format!("Upload failed: {}", e), StatusCode::INTERNAL_SERVER_ERROR).into_response();
                }
            }
        }
    }

    ApiError("No thumbnail field found in multipart request".to_string(), StatusCode::BAD_REQUEST).into_response()
}

/// Get Movie Thumbnail
/// Serves the thumbnail image from MinIO
#[utoipa::path(
    get,
    path = "/api/v1/movies/{id}/thumbnail",
    params(("id" = Uuid, Path, description = "Movie ID")),
    responses(
        (status = 200, description = "Success", body = Vec<u8>),
        (status = 404, description = "Not Found")
    ),
    tag = "Content"
)]
pub async fn get_movie_thumbnail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // 1. Get Movie and Thumbnail Key
    use crate::modules::content::repository::ContentRepository;
    
    let movie_opt = ContentRepository::get_movie_by_id(&state.db, id).await.unwrap_or(None);
    let movie = match movie_opt {
        Some(m) => m,
        None => return ApiError("Movie not found".to_string(), StatusCode::NOT_FOUND).into_response(),
    };

    let key = match movie.thumbnail_url {
        Some(k) => k,
        None => return ApiError("Movie has no thumbnail".to_string(), StatusCode::NOT_FOUND).into_response(),
    };

    // 2. Fetch from MinIO (Thumbs bucket)
    // We need to use the thumbnails bucket.
    // Assuming `state.storage.get_object` uses `self.bucket`.
    // We need to target the thumbnails bucket.
    
    // Either method on StorageService to override bucket, or clone.
    // Let's create `get_thumbnail_object` in `StorageService` or just use cloned struct hack again.
    let mut storage_for_thumb = state.storage.clone();
    storage_for_thumb.bucket = state.config.minio_bucket_thumbnails.clone();
    
    match storage_for_thumb.get_object(&key).await {
        Ok(bytes) => {
            // Determine content type
            let content_type = mime_guess::from_path(&key).first_or_octet_stream().to_string();
            
            ([(axum::http::header::CONTENT_TYPE, content_type)], bytes).into_response()
        },
        Err(e) => {
            tracing::error!("Failed to fetch thumbnail {}: {}", key, e);
            ApiError("Thumbnail not found in storage".to_string(), StatusCode::NOT_FOUND).into_response()
        }
    }
}

// --- UPDATE & DELETE HANDLERS ---

#[utoipa::path(
    put,
    path = "/api/v1/movies/{id}",
    params(("id" = Uuid, Path, description = "Movie ID")),
    request_body = UpdateMovieRequest,
    responses(
        (status = 200, description = "Movie Updated", body = ApiResponse<MovieResponse>),
        (status = 404, description = "Not Found")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn update_movie(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMovieRequest>,
) -> impl IntoResponse {
    match ContentService::update_movie(state, id, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Movie updated").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/movies/{id}",
    params(("id" = Uuid, Path, description = "Movie ID")),
    responses(
        (status = 200, description = "Movie Deleted"),
        (status = 404, description = "Not Found")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn delete_movie(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ContentService::delete_movie(state, id).await {
        Ok(_) => ApiSuccess(ApiResponse::success((), "Movie deleted").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/series/{id}",
    params(("id" = Uuid, Path, description = "Series ID")),
    request_body = UpdateSeriesRequest,
    responses(
        (status = 200, description = "Series Updated", body = ApiResponse<SeriesResponse>),
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn update_series(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSeriesRequest>,
) -> impl IntoResponse {
    match ContentService::update_series(state, id, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Series updated").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/series/{id}",
    params(("id" = Uuid, Path, description = "Series ID")),
    responses(
        (status = 200, description = "Series Deleted")
    ),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn delete_series(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ContentService::delete_series(state, id).await {
        Ok(_) => ApiSuccess(ApiResponse::success((), "Series deleted").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

// Seasons & Episodes...
#[utoipa::path(
    put,
    path = "/api/v1/seasons/{id}",
    params(("id" = Uuid, Path, description = "Season ID")),
    request_body = UpdateSeasonRequest,
    responses((status = 200, description = "Updated")),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn update_season(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSeasonRequest>,
) -> impl IntoResponse {
    match ContentService::update_season(state, id, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Season updated").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/seasons/{id}",
    params(("id" = Uuid, Path, description = "Season ID")),
    responses((status = 200, description = "Deleted")),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn delete_season(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ContentService::delete_season(state, id).await {
        Ok(_) => ApiSuccess(ApiResponse::success((), "Season deleted").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/episodes/{id}",
    params(("id" = Uuid, Path, description = "Episode ID")),
    request_body = UpdateEpisodeRequest,
    responses((status = 200, description = "Updated")),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn update_episode(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEpisodeRequest>,
) -> impl IntoResponse {
    match ContentService::update_episode(state, id, req).await {
        Ok(res) => ApiSuccess(ApiResponse::success(res, "Episode updated").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/episodes/{id}",
    params(("id" = Uuid, Path, description = "Episode ID")),
    responses((status = 200, description = "Deleted")),
    tag = "Content",
    security(("bearer_auth" = []))
)]
pub async fn delete_episode(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ContentService::delete_episode(state, id).await {
        Ok(_) => ApiSuccess(ApiResponse::success((), "Episode deleted").into(), StatusCode::OK).into_response(),
        Err(e) => ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}
