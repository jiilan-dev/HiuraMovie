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
    ),
    components(
        schemas(RegisterRequest, LoginRequest, AuthResponse, UserResponse)
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct ApiDoc;

// Define SecurityScheme separately if needed, or inline in derive macro.
// Utoipa 4+ uses modifier for security schemes, let's stick to simple derive for now.
// For Bearer auth:
use utoipa::Modify;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder, SecuritySchemeType};

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
