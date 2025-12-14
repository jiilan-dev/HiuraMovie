use axum::Router;
use crate::state::AppState;

pub fn configure_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", api_routes())
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
}
