use axum::{Router, extract::DefaultBodyLimit};
use crate::state::AppState;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

pub async fn create_app(state: AppState) -> Router {
    crate::routes::configure_routes(state.clone())
        .layer(DefaultBodyLimit::disable()) // Allow unlimited body size for video uploads
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(state)
}
