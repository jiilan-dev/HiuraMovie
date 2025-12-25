use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use crate::state::AppState;
use uuid::Uuid;
use futures_util::TryStreamExt;
use std::io;

/// Stream video content with support for Range requests
/// Proxies the stream from S3/MinIO to the client efficiently
#[utoipa::path(
    get,
    path = "/api/v1/movies/{id}/stream",
    params(
        ("id" = Uuid, Path, description = "Movie ID")
    ),
    responses(
        (status = 200, description = "Stream Content"),
        (status = 206, description = "Partial Content"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Content"
)]
pub async fn stream_movie(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 1. Get movie to find video_url
    let movie = match crate::modules::content::repository::ContentRepository::get_movie_by_id(&state.db, id).await {
        Ok(Some(m)) => m,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("Database Error: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    
    let video_key = match movie.video_url {
        Some(k) => k,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    // 2. Parse Range header
    let range_header = headers.get(header::RANGE)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // 3. Prepare S3 Request
    let mut req = state.storage.client
        .get_object()
        .bucket(&state.config.minio_bucket)
        .key(video_key);
    
    if let Some(r) = range_header {
        req = req.range(r);
    }
    
    // 4. Send Request to S3
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("S3 Error: {}", e);
            // Handle specific S3 errors like 404
             return StatusCode::NOT_FOUND.into_response(); 
        }
    };
    
    // 5. Build Response
    let mut builder = axum::response::Response::builder();
    
    // Copy relevant headers
    if let Some(ct) = resp.content_type() {
        builder = builder.header(header::CONTENT_TYPE, ct);
    } else {
         builder = builder.header(header::CONTENT_TYPE, "video/mp4");
    }
    
    if let Some(cl) = resp.content_length() {
        builder = builder.header(header::CONTENT_LENGTH, cl);
    }
    
    if let Some(cr) = resp.content_range() {
         builder = builder.header(header::CONTENT_RANGE, cr).status(StatusCode::PARTIAL_CONTENT);
    } else {
         builder = builder.header(header::ACCEPT_RANGES, "bytes").status(StatusCode::OK);
    }
    
    if let Some(et) = resp.e_tag() {
        builder = builder.header(header::ETAG, et);
    }

    // 6. Create Stream Body
    use tokio_util::io::ReaderStream;
    
    let reader = resp.body.into_async_read();
    let stream = ReaderStream::new(reader);
    
    let body = Body::from_stream(stream);

    builder.body(body).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
