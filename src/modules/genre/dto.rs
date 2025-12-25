use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct CreateGenreRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateGenreRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GenreResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}
