use axum::Router;

pub fn configure_routes() -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
}

fn api_routes() -> Router {
    Router::new()
        // We will add module routes here later
        .route("/health", axum::routing::get(|| async { "ok" }))
}
