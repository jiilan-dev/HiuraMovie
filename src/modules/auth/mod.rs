use axum::Router;
use axum::routing::post;
use crate::state::AppState;
use axum::middleware;

pub mod dto;
pub mod handler;
pub mod model;
pub mod repository;
pub mod service;

pub fn router(state: AppState) -> axum::Router<AppState> {
    let public_routes = Router::new()
        .route("/register", post(handler::register))
        .route("/login", post(handler::login))
        .route("/refresh", post(handler::refresh));

    let protected_routes = Router::new()
        .route("/logout", post(handler::logout))
        .route("/me", axum::routing::get(handler::get_me))
        .route_layer(middleware::from_fn_with_state(
            state,
            crate::middleware::auth::auth_middleware
        ));

    public_routes.merge(protected_routes)
}
