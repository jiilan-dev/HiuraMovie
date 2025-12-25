use super::dto::{CreateGenreRequest, GenreResponse, UpdateGenreRequest};
use super::repository::GenreRepository;
use crate::state::AppState;
use anyhow::Result;
use uuid::Uuid;

pub struct GenreService;

impl GenreService {
    pub async fn create(state: AppState, req: CreateGenreRequest) -> Result<GenreResponse> {
        let genre = GenreRepository::create(&state.db, &req.name, &req.slug).await?;
        
        Ok(GenreResponse {
            id: genre.id,
            name: genre.name,
            slug: genre.slug,
        })
    }

    pub async fn find_all(state: AppState) -> Result<Vec<GenreResponse>> {
        let genres = GenreRepository::find_all(&state.db).await?;
        
        Ok(genres
            .into_iter()
            .map(|g| GenreResponse {
                id: g.id,
                name: g.name,
                slug: g.slug,
            })
            .collect())
    }

    pub async fn find_by_id(state: AppState, id: Uuid) -> Result<GenreResponse> {
        let genre = GenreRepository::find_by_id(&state.db, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Genre not found"))?;
            
        Ok(GenreResponse {
            id: genre.id,
            name: genre.name,
            slug: genre.slug,
        })
    }

    pub async fn update(state: AppState, id: Uuid, req: UpdateGenreRequest) -> Result<GenreResponse> {
        let genre = GenreRepository::update(&state.db, id, req.name, req.slug).await?;
        
        Ok(GenreResponse {
            id: genre.id,
            name: genre.name,
            slug: genre.slug,
        })
    }

    pub async fn delete(state: AppState, id: Uuid) -> Result<()> {
        GenreRepository::delete(&state.db, id).await?;
        Ok(())
    }
}
