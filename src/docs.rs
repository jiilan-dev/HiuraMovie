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
    ),
    components(
        schemas(
            RegisterRequest, LoginRequest, AuthResponse, UserResponse,
            crate::modules::genre::dto::CreateGenreRequest,
            crate::modules::genre::dto::UpdateGenreRequest,
            crate::modules::genre::dto::GenreResponse,
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Content", description = "Video Content Management")
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
