use axum::Router;

pub async fn create_app() -> Router {
    Router::new()
        .merge(crate::routes::configure_routes())
}
