use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGenreRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateGenreRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
}

use crate::modules::genre::model::Genre;

#[derive(Debug, Serialize, ToSchema)] // Removed From, Into
pub struct GenreResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

impl From<Genre> for GenreResponse {
    fn from(g: Genre) -> Self {
        Self {
            id: g.id,
            name: g.name,
            slug: g.slug,
        }
    }
}
