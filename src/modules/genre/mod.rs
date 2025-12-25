use axum::Router;
use axum::routing::{get, post};
use crate::state::AppState;
use axum::middleware;

pub mod dto;
pub mod handler;
pub mod model;
pub mod repository;
pub mod service;

pub fn router(state: AppState) -> axum::Router<AppState> {
    let public_routes = Router::new()
        .route("/", get(handler::list_genres))
        .route("/{id}", get(handler::get_genre));

    let protected_routes = Router::new()
        .route("/", post(handler::create_genre))
        .route("/{id}",  axum::routing::put(handler::update_genre).delete(handler::delete_genre))
        .route_layer(middleware::from_fn(crate::middleware::role::admin_guard))
        .route_layer(middleware::from_fn_with_state(
            state,
            crate::middleware::auth::auth_middleware
        ));

    public_routes.merge(protected_routes)
}
