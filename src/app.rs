use axum::Router;
use crate::state::AppState;

pub async fn create_app(state: AppState) -> Router {
    crate::routes::configure_routes(state.clone())
        .with_state(state)
}
