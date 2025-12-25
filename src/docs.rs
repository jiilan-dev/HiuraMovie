use utoipa::OpenApi;
use crate::modules::auth::dto::*;
use crate::modules::auth::handler::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::modules::auth::handler::register,
        crate::modules::auth::handler::login,
        crate::modules::auth::handler::logout,
        crate::modules::auth::handler::refresh,
        crate::modules::genre::handler::list_genres,
        crate::modules::genre::handler::create_genre,
        crate::modules::genre::handler::get_genre,
        crate::modules::genre::handler::update_genre,
        crate::modules::genre::handler::delete_genre,
        // Content
        crate::modules::content::handler::create_movie,
        crate::modules::content::handler::list_movies,
        crate::modules::content::handler::get_movie,
        crate::modules::content::handler::upload_movie_video,
        crate::modules::content::handler::create_series,
        crate::modules::content::handler::list_series,
        crate::modules::content::handler::get_series,
        crate::modules::content::handler::create_season,
        crate::modules::content::handler::create_episode,
        // Update & Delete
        crate::modules::content::handler::update_movie,
        crate::modules::content::handler::delete_movie,
        crate::modules::content::handler::update_series,
        crate::modules::content::handler::delete_series,
        crate::modules::content::handler::update_season,
        crate::modules::content::handler::delete_season,
        crate::modules::content::handler::update_episode,
        crate::modules::content::handler::delete_episode,
        // Streaming
        crate::modules::content::stream_handler::stream_movie,
    ),
    components(
        schemas(
            crate::common::response::ApiResponse<String>,
            crate::modules::auth::dto::LoginRequest,
            crate::modules::auth::dto::RegisterRequest,
            crate::modules::auth::dto::AuthResponse,
            crate::modules::auth::dto::UserResponse,
            // Genre
            crate::modules::genre::dto::CreateGenreRequest,
            crate::modules::genre::dto::UpdateGenreRequest,
            crate::modules::genre::dto::GenreResponse,
            crate::modules::genre::model::Genre,
            // Content
            crate::modules::content::dto::CreateMovieRequest,
            crate::modules::content::dto::UpdateMovieRequest,
            crate::modules::content::dto::MovieResponse,
            crate::modules::content::dto::CreateSeriesRequest,
            crate::modules::content::dto::UpdateSeriesRequest,
            crate::modules::content::dto::SeriesResponse,
            crate::modules::content::dto::SeriesListResponse,
            crate::modules::content::dto::CreateSeasonRequest,
            crate::modules::content::dto::UpdateSeasonRequest,
            crate::modules::content::dto::SeasonResponse,
            crate::modules::content::dto::CreateEpisodeRequest,
            crate::modules::content::dto::UpdateEpisodeRequest,
            crate::modules::content::model::Movie,
            crate::modules::content::model::Series,
            crate::modules::content::model::Season,
            crate::modules::content::model::Episode,
            crate::modules::content::model::ContentStatus,
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Genre", description = "Genre management endpoints"),
        (name = "Content", description = "Movie and Series management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

// Define SecurityScheme separately if needed, or inline in derive macro.
// Utoipa 4+ uses modifier for security schemes, let's stick to simple derive for now.
// For Bearer auth:
use utoipa::Modify;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
