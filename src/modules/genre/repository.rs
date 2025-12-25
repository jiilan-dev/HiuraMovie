use super::model::Genre;
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

pub struct GenreRepository;

impl GenreRepository {
    pub async fn create(pool: &PgPool, name: &str, slug: &str) -> Result<Genre> {
        let genre = sqlx::query_as!(
            Genre,
            r#"
            INSERT INTO genres (name, slug)
            VALUES ($1, $2)
            RETURNING id, name, slug, created_at, updated_at
            "#,
            name,
            slug
        )
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to create genre: {}", e))?;

        Ok(genre)
    }

    pub async fn find_all(pool: &PgPool) -> Result<Vec<Genre>> {
        let genres = sqlx::query_as!(
            Genre,
            r#"
            SELECT id, name, slug, created_at, updated_at
            FROM genres
            ORDER BY name ASC
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch genres: {}", e))?;

        Ok(genres)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Genre>> {
        let genre = sqlx::query_as!(
            Genre,
            r#"
            SELECT id, name, slug, created_at, updated_at
            FROM genres
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch genre: {}", e))?;

        Ok(genre)
    }

    pub async fn update(pool: &PgPool, id: Uuid, name: Option<String>, slug: Option<String>) -> Result<Genre> {
        let mut tx = pool.begin().await?;

        // Checking existence is implicitly done by update returning row
        // Dynamic query building is tricky with sqlx macros, so we might check fields
        // Since we have few fields, we can do coalescing or just fetch first. 
        // For simplicity let's fetch first.
        let _current = sqlx::query!("SELECT id FROM genres WHERE id = $1", id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| anyhow!("Genre not found"))?;

        let genre = sqlx::query_as!(
            Genre,
            r#"
            UPDATE genres
            SET 
                name = COALESCE($1, name),
                slug = COALESCE($2, slug),
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, name, slug, created_at, updated_at
            "#,
            name,
            slug,
            id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to update genre: {}", e))?;

        tx.commit().await?;
        Ok(genre)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query!("DELETE FROM genres WHERE id = $1", id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Genre not found"));
        }

        Ok(())
    }
}
