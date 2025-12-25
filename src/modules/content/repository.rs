use sqlx::PgPool;
use uuid::Uuid;
use super::model::{Movie, Series, Season, Episode};
use crate::modules::genre::model::Genre;
use anyhow::{Result, anyhow};

pub struct ContentRepository;

impl ContentRepository {
    // --- MOVIE ---
    
    pub async fn create_movie(
        pool: &PgPool,
        title: &str,
        slug: &str,
        description: Option<String>,
        release_year: Option<i32>,
        duration_seconds: Option<i32>,
    ) -> Result<Movie> {
        let movie = sqlx::query_as!(
            Movie,
            r#"
            INSERT INTO movies (title, slug, description, release_year, duration_seconds)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            title,
            slug,
            description,
            release_year,
            duration_seconds
        )
        .fetch_one(pool)
        .await?;

        Ok(movie)
    }

    pub async fn update_movie_video_url(
        pool: &PgPool,
        id: Uuid,
        video_url: &str,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE movies SET video_url = $1, status = 'READY', updated_at = NOW() WHERE id = $2",
            video_url,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_movie_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Movie>> {
        let movie = sqlx::query_as!(
            Movie,
            "SELECT * FROM movies WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(movie)
    }

    pub async fn get_movie_genres(pool: &PgPool, movie_id: Uuid) -> Result<Vec<Genre>> {
        let genres = sqlx::query_as!(
            Genre,
            r#"
            SELECT g.* 
            FROM genres g
            JOIN content_genres cg ON g.id = cg.genre_id
            WHERE cg.movie_id = $1
            "#,
            movie_id
        )
        .fetch_all(pool)
        .await?;
        Ok(genres)
    }

    pub async fn link_movie_genres(pool: &PgPool, movie_id: Uuid, genre_ids: &[Uuid]) -> Result<()> {
        // Start transaction manually if needed, or query one by one. 
        // For simple inserts, UNNEST is efficient.
        sqlx::query!(
            r#"
            INSERT INTO content_genres (movie_id, genre_id)
            SELECT $1, unnest($2::uuid[])
            ON CONFLICT DO NOTHING
            "#,
            movie_id,
            genre_ids
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn list_movies(pool: &PgPool) -> Result<Vec<Movie>> {
        let movies = sqlx::query_as!(
            Movie,
            "SELECT * FROM movies ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;
        Ok(movies)
    }
    
    // --- SERIES ---

    pub async fn create_series(
        pool: &PgPool,
        title: &str,
        slug: &str,
        description: Option<String>,
        release_year: Option<i32>,
    ) -> Result<Series> {
        let series = sqlx::query_as!(
            Series,
            r#"
            INSERT INTO series (title, slug, description, release_year)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            title,
            slug,
            description,
            release_year
        )
        .fetch_one(pool)
        .await?;
        Ok(series)
    }

    pub async fn get_series_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Series>> {
        let series = sqlx::query_as!(
            Series,
            "SELECT * FROM series WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(series)
    }

    pub async fn link_series_genres(pool: &PgPool, series_id: Uuid, genre_ids: &[Uuid]) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO content_genres (series_id, genre_id)
            SELECT $1, unnest($2::uuid[])
            ON CONFLICT DO NOTHING
            "#,
            series_id,
            genre_ids
        )
        .execute(pool)
        .await?;
        Ok(())
    }
    
    pub async fn get_series_genres(pool: &PgPool, series_id: Uuid) -> Result<Vec<Genre>> {
        let genres = sqlx::query_as!(
            Genre,
            r#"
            SELECT g.* 
            FROM genres g
            JOIN content_genres cg ON g.id = cg.genre_id
            WHERE cg.series_id = $1
            "#,
            series_id
        )
        .fetch_all(pool)
        .await?;
        Ok(genres)
    }

    pub async fn list_series(pool: &PgPool) -> Result<Vec<Series>> {
        let series = sqlx::query_as!(
            Series,
            "SELECT * FROM series ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;
        Ok(series)
    }

    // --- SEASONS ---

    pub async fn create_season(
        pool: &PgPool,
        series_id: Uuid,
        season_number: i32,
        title: Option<String>,
    ) -> Result<Season> {
        let season = sqlx::query_as!(
            Season,
            r#"
            INSERT INTO seasons (series_id, season_number, title)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            series_id,
            season_number,
            title
        )
        .fetch_one(pool)
        .await?;
        Ok(season)
    }
    
    pub async fn get_series_seasons(pool: &PgPool, series_id: Uuid) -> Result<Vec<Season>> {
        let seasons = sqlx::query_as!(
            Season,
            "SELECT * FROM seasons WHERE series_id = $1 ORDER BY season_number ASC",
            series_id
        )
        .fetch_all(pool)
        .await?;
        Ok(seasons)
    }

    pub async fn get_season_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Season>> {
        let season = sqlx::query_as!(
            Season,
            "SELECT * FROM seasons WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await?;
        Ok(season)
    }

    // --- EPISODES ---

    pub async fn create_episode(
        pool: &PgPool,
        season_id: Uuid,
        episode_number: i32,
        title: Option<String>,
        description: Option<String>,
        duration_seconds: Option<i32>,
    ) -> Result<Episode> {
        let episode = sqlx::query_as!(
            Episode,
            r#"
            INSERT INTO episodes (season_id, episode_number, title, description, duration_seconds)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            season_id,
            episode_number,
            title,
            description,
            duration_seconds
        )
        .fetch_one(pool)
        .await?;
        Ok(episode)
    }

    pub async fn get_season_episodes(pool: &PgPool, season_id: Uuid) -> Result<Vec<Episode>> {
        let episodes = sqlx::query_as!(
            Episode,
            "SELECT * FROM episodes WHERE season_id = $1 ORDER BY episode_number ASC",
            season_id
        )
        .fetch_all(pool)
        .await?;
        Ok(episodes)
    }
    // --- MOVIE UPDATES ---

    pub async fn update_movie(
        pool: &PgPool,
        id: Uuid,
        title: Option<String>,
        description: Option<String>,
        release_year: Option<i32>,
    ) -> Result<Movie> {
        let movie = sqlx::query_as!(
            Movie,
            r#"
            UPDATE movies 
            SET 
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                release_year = COALESCE($3, release_year),
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
            title,
            description,
            release_year,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(movie)
    }

    pub async fn delete_movie(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM movies WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // --- SERIES UPDATES ---

    pub async fn update_series(
        pool: &PgPool,
        id: Uuid,
        title: Option<String>,
        description: Option<String>,
        release_year: Option<i32>,
    ) -> Result<Series> {
        let series = sqlx::query_as!(
            Series,
            r#"
            UPDATE series 
            SET 
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                release_year = COALESCE($3, release_year),
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
            title,
            description,
            release_year,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(series)
    }

    pub async fn delete_series(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM series WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // --- SEASON UPDATES ---

    pub async fn update_season(
        pool: &PgPool,
        id: Uuid,
        title: Option<String>,
        season_number: Option<i32>,
    ) -> Result<Season> {
        let season = sqlx::query_as!(
            Season,
            r#"
            UPDATE seasons 
            SET 
                title = COALESCE($1, title),
                season_number = COALESCE($2, season_number),
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
            title,
            season_number,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(season)
    }
    
    pub async fn delete_season(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM seasons WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // --- EPISODE UPDATES ---

    pub async fn update_episode(
        pool: &PgPool,
        id: Uuid,
        title: Option<String>,
        description: Option<String>,
        episode_number: Option<i32>,
        duration_seconds: Option<i32>,
    ) -> Result<Episode> {
        let episode = sqlx::query_as!(
            Episode,
            r#"
            UPDATE episodes 
            SET 
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                episode_number = COALESCE($3, episode_number),
                duration_seconds = COALESCE($4, duration_seconds),
                updated_at = NOW()
            WHERE id = $5
            RETURNING *
            "#,
            title,
            description,
            episode_number,
            duration_seconds,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(episode)
    }

    pub async fn delete_episode(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM episodes WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // --- GENRE GENERIC LINKing ---
    pub async fn clear_content_genres(pool: &PgPool, movie_id: Option<Uuid>, series_id: Option<Uuid>) -> Result<()> {
        if let Some(mid) = movie_id {
            sqlx::query!("DELETE FROM content_genres WHERE movie_id = $1", mid)
                .execute(pool)
                .await?;
        }
        if let Some(sid) = series_id {
             sqlx::query!("DELETE FROM content_genres WHERE series_id = $1", sid)
                .execute(pool)
                .await?;
        }
        Ok(())
    }
}
