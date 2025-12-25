use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use super::model::{Movie, Series, Season, Episode};
use crate::modules::genre::dto::GenreResponse;

// --- MOVIE DTOs ---

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMovieRequest {
    pub title: String,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub genre_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMovieRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub genre_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MovieResponse {
    pub movie: Movie,
    pub genres: Vec<GenreResponse>,
}

// --- SERIES DTOs ---

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSeriesRequest {
    pub title: String,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub genre_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSeriesRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub genre_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesResponse {
    pub series: Series,
    pub genres: Vec<GenreResponse>,
    pub seasons: Vec<SeasonResponse>, // Nested full structure
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeriesListResponse {
    pub series: Series,
    pub genres: Vec<GenreResponse>,
}

// --- SEASON DTOs ---

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSeasonRequest {
    pub series_id: Uuid,
    pub season_number: i32,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSeasonRequest {
    pub title: Option<String>,
    pub season_number: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeasonResponse {
    pub season: Season,
    pub episodes: Vec<Episode>,
}

// --- EPISODE DTOs ---

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEpisodeRequest {
    pub season_id: Uuid,
    pub episode_number: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEpisodeRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub episode_number: Option<i32>,
    pub duration_seconds: Option<i32>,
}
