use axum::Router;
use crate::state::AppState;

pub async fn create_app(state: AppState) -> Router {
    crate::routes::configure_routes()
        .with_state(state)
}
