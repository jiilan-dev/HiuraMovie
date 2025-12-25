use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::docs::ApiDoc;
use axum::Router;
use crate::state::AppState;

use tower_http::cors::{Any, CorsLayer};

pub fn configure_routes(state: AppState) -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_routes())
        .nest("/api/v1/auth", crate::modules::auth::router(state.clone()))
        .nest("/api/v1/genres", crate::modules::genre::router(state.clone()))
        .nest("/api/v1", crate::modules::content::router(state))
        .layer(cors)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
}
