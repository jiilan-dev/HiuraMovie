use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)] // No FromRow here, usually custom queries
pub enum ContentStatus {
    DRAFT,
    PROCESSING,
    READY,
    FAILED,
}

impl ToString for ContentStatus {
    fn to_string(&self) -> String {
        match self {
            ContentStatus::DRAFT => "DRAFT".to_string(),
            ContentStatus::PROCESSING => "PROCESSING".to_string(),
            ContentStatus::READY => "READY".to_string(),
            ContentStatus::FAILED => "FAILED".to_string(),
        }
    }
}

impl From<String> for ContentStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "PROCESSING" => ContentStatus::PROCESSING,
            "READY" => ContentStatus::READY,
            "FAILED" => ContentStatus::FAILED,
            _ => ContentStatus::DRAFT,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Movie {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub video_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub subtitle_url: Option<String>,
    pub release_year: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub rating: Option<f64>, // Changed from f32 to f64 for Postgres compatibility
    pub views: Option<i32>,
    pub status: Option<String>, // Stored as string in DB
    #[schema(value_type = String, format = Date)]
    pub created_at: OffsetDateTime,
    #[schema(value_type = String, format = Date)]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Series {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub release_year: Option<i32>,
    pub rating: Option<f64>,
    #[schema(value_type = String, format = Date)]
    pub created_at: OffsetDateTime,
    #[schema(value_type = String, format = Date)]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Season {
    pub id: Uuid,
    pub series_id: Uuid,
    pub season_number: i32,
    pub title: Option<String>,
    #[schema(value_type = String, format = Date)]
    pub created_at: OffsetDateTime,
    #[schema(value_type = String, format = Date)]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Episode {
    pub id: Uuid,
    pub season_id: Uuid,
    pub episode_number: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub video_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub subtitle_url: Option<String>,
    pub duration_seconds: Option<i32>,
    pub views: Option<i32>,
    pub status: Option<String>,
    #[schema(value_type = String, format = Date)]
    pub created_at: OffsetDateTime,
    #[schema(value_type = String, format = Date)]
    pub updated_at: OffsetDateTime,
}

// For query results joining genres
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ContentGenreLink {
    pub genre_id: Uuid,
    pub genre_name: String,
}
