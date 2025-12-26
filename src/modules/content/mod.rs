use axum::Router;
use axum::routing::post;
use crate::state::AppState;
use axum::middleware;

pub mod handler;
pub mod stream_handler; // Added
pub mod events;
pub mod dto;
pub mod model;
pub mod repository;
pub mod service;

pub fn router(state: AppState) -> axum::Router<AppState> {
    
    let public_routes = Router::new()
        .route("/movies", axum::routing::get(handler::list_movies))
        .route("/movies/{id}", axum::routing::get(handler::get_movie))
        .route("/movies/{id}/progress", axum::routing::get(handler::get_movie_transcode_progress))
        .route("/movies/{id}/stream", axum::routing::get(stream_handler::stream_movie))
        .route("/movies/{id}/thumbnail", axum::routing::get(handler::get_movie_thumbnail))
        .route("/movies/{id}/subtitle", axum::routing::get(handler::get_movie_subtitle))
        .route("/episodes/{id}/stream", axum::routing::get(stream_handler::stream_episode))
        .route("/episodes/{id}/progress", axum::routing::get(handler::get_episode_transcode_progress))
        .route("/episodes/{id}/subtitle", axum::routing::get(handler::get_episode_subtitle))
        .route("/series", axum::routing::get(handler::list_series))
        .route("/series/{id}", axum::routing::get(handler::get_series))
        .route("/series/{id}/thumbnail", axum::routing::get(handler::get_series_thumbnail));

    let protected_routes = Router::new()
        .route("/movies", post(handler::create_movie))
        .route("/movies/{id}/upload", post(handler::upload_movie_video))
        .route("/movies/{id}/upload-thumbnail", post(handler::upload_movie_thumbnail))
        .route("/movies/{id}", axum::routing::put(handler::update_movie).delete(handler::delete_movie))
        
        .route("/series", post(handler::create_series))
        .route("/series/{id}/upload-thumbnail", post(handler::upload_series_thumbnail))
        .route("/series/{id}", axum::routing::put(handler::update_series).delete(handler::delete_series))
        
        .route("/seasons", post(handler::create_season))
        .route("/seasons/{id}", axum::routing::put(handler::update_season).delete(handler::delete_season))
        
        .route("/episodes", post(handler::create_episode))
        .route("/episodes/{id}", axum::routing::put(handler::update_episode).delete(handler::delete_episode))
        .route("/episodes/{id}/upload", post(handler::upload_episode_video))
        .route("/episodes/{id}/upload-thumbnail", post(handler::upload_episode_thumbnail))
        .route_layer(middleware::from_fn(crate::middleware::role::admin_guard))
        .route_layer(middleware::from_fn_with_state(
            state,
            crate::middleware::auth::auth_middleware
        ));

    public_routes.merge(protected_routes)
}
