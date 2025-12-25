use super::dto::{
    CreateMovieRequest, CreateSeriesRequest, CreateSeasonRequest, CreateEpisodeRequest,
    UpdateMovieRequest, UpdateSeriesRequest, UpdateSeasonRequest, UpdateEpisodeRequest,
    MovieResponse, SeriesResponse, SeriesListResponse, SeasonResponse
};
use super::repository::ContentRepository;
use crate::modules::genre::dto::GenreResponse;
use crate::state::AppState;
use anyhow::{Result, anyhow};
use uuid::Uuid;
// use slug::slugify; // Removed unused import

pub struct ContentService;

impl ContentService {
    fn generate_slug(title: &str) -> String {
        // For now let's just use a simple replace. Ideally use `slug` crate.
        title.to_lowercase()
            .replace(" ", "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect()
    }

    // --- MOVIE ---

    pub async fn create_movie(state: AppState, req: CreateMovieRequest) -> Result<MovieResponse> {
        let slug = format!("{}-{}", Self::generate_slug(&req.title), Uuid::new_v4().as_simple().to_string()[..6].to_string());
        
        let movie = ContentRepository::create_movie(
            &state.db,
            &req.title,
            &slug,
            req.description,
            req.release_year,
            req.duration_seconds,
        ).await?;

        if !req.genre_ids.is_empty() {
            ContentRepository::link_movie_genres(&state.db, movie.id, &req.genre_ids).await?;
        }
        
        // Fetch full data for response
        let genres = ContentRepository::get_movie_genres(&state.db, movie.id).await?;
        let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();

        Ok(MovieResponse {
            movie,
            genres: genre_dtos,
        })
    }
    
    pub async fn list_movies(state: AppState) -> Result<Vec<MovieResponse>> {
        let movies = ContentRepository::list_movies(&state.db).await?;
        
        let mut responses = Vec::new();
        for movie in movies {
             let genres = ContentRepository::get_movie_genres(&state.db, movie.id).await?;
             let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();
             responses.push(MovieResponse { movie, genres: genre_dtos });
        }
        
        Ok(responses)
    }

    pub async fn get_movie(state: AppState, id: Uuid) -> Result<MovieResponse> {
        let movie = ContentRepository::get_movie_by_id(&state.db, id).await?
            .ok_or(anyhow!("Movie not found"))?;
            
        let genres = ContentRepository::get_movie_genres(&state.db, movie.id).await?;
        let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();

        Ok(MovieResponse {
            movie,
            genres: genre_dtos,
        })
    }

    // --- SERIES ---

    pub async fn create_series(state: AppState, req: CreateSeriesRequest) -> Result<SeriesResponse> {
        let slug = format!("{}-{}", Self::generate_slug(&req.title), Uuid::new_v4().as_simple().to_string()[..6].to_string());
        
        let series = ContentRepository::create_series(
            &state.db,
            &req.title,
            &slug,
            req.description,
            req.release_year,
        ).await?;

        if !req.genre_ids.is_empty() {
            ContentRepository::link_series_genres(&state.db, series.id, &req.genre_ids).await?;
        }
        
        let genres = ContentRepository::get_series_genres(&state.db, series.id).await?;
        let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();

        Ok(SeriesResponse {
            series,
            genres: genre_dtos,
            seasons: vec![],
        })
    }

    pub async fn list_series(state: AppState) -> Result<Vec<SeriesListResponse>> {
        let series_list = ContentRepository::list_series(&state.db).await?;
        
        let mut responses = Vec::new();
        for s in series_list {
             let genres = ContentRepository::get_series_genres(&state.db, s.id).await?;
             let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();
             responses.push(SeriesListResponse { series: s, genres: genre_dtos });
        }
        
        Ok(responses)
    }
    
    pub async fn get_series(state: AppState, id: Uuid) -> Result<SeriesResponse> {
        let series = ContentRepository::get_series_by_id(&state.db, id).await?
            .ok_or(anyhow!("Series not found"))?;
            
        let genres = ContentRepository::get_series_genres(&state.db, series.id).await?;
        let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();
        
        // Get seasons and episodes
        let season_models = ContentRepository::get_series_seasons(&state.db, series.id).await?;
        let mut season_responses = Vec::new();
        
        for season in season_models {
            let episodes = ContentRepository::get_season_episodes(&state.db, season.id).await?;
            season_responses.push(SeasonResponse {
                season,
                episodes
            });
        }

        Ok(SeriesResponse {
            series,
            genres: genre_dtos,
            seasons: season_responses,
        })
    }



    // --- SEASONS & EPISODES ---

    pub async fn create_season(state: AppState, req: CreateSeasonRequest) -> Result<SeasonResponse> {
        // Verify series exists
        if ContentRepository::get_series_by_id(&state.db, req.series_id).await?.is_none() {
            return Err(anyhow!("Series not found"));
        }

        let season = ContentRepository::create_season(
            &state.db,
            req.series_id,
            req.season_number,
            req.title,
        ).await?;

        Ok(SeasonResponse {
            season,
            episodes: vec![],
        })
    }

    pub async fn create_episode(state: AppState, req: CreateEpisodeRequest) -> Result<super::model::Episode> {
        // Verify season exists
        if ContentRepository::get_season_by_id(&state.db, req.season_id).await?.is_none() {
            return Err(anyhow!("Season not found"));
        }

        let episode = ContentRepository::create_episode(
            &state.db,
            req.season_id,
            req.episode_number,
            req.title,
            req.description,
            req.duration_seconds,
        ).await?;
        
        Ok(episode)
    }

    pub async fn update_episode(state: AppState, id: Uuid, req: UpdateEpisodeRequest) -> Result<super::model::Episode> {
        let episode = ContentRepository::update_episode(
            &state.db,
            id,
            req.title,
            req.description,
            req.episode_number,
            req.duration_seconds
        ).await?;
        Ok(episode)
    }

    pub async fn delete_episode(state: AppState, id: Uuid) -> Result<()> {
        ContentRepository::delete_episode(&state.db, id).await
    }
}

impl ContentService {
   // ... previous methods ...

    // --- MOVIE UPDATES ---
    pub async fn complete_movie_upload(state: AppState, id: Uuid, video_key: String) -> Result<()> {
        let video_url = video_key; // In a real app with CDN, this would be full URL. For now relative key.
        
        sqlx::query!(
            "UPDATE movies SET video_url = $1, status = 'READY', updated_at = NOW() WHERE id = $2",
            video_url,
            id
        )
        .execute(&state.db)
        .await?;
        
        Ok(())
    }

    pub async fn complete_movie_thumbnail_upload(state: AppState, id: Uuid, thumbnail_key: String) -> Result<()> {
        // Thumbnail URL handling
        let thumbnail_url = thumbnail_key;
        
        sqlx::query!(
            "UPDATE movies SET thumbnail_url = $1, updated_at = NOW() WHERE id = $2",
            thumbnail_url,
            id
        )
        .execute(&state.db)
        .await?;
        
        Ok(())
    }
    pub async fn update_movie(state: AppState, id: Uuid, req: UpdateMovieRequest) -> Result<MovieResponse> {
        let movie = ContentRepository::update_movie(
            &state.db,
            id,
            req.title,
            req.description,
            req.release_year,
        ).await?;

        if let Some(gids) = req.genre_ids {
            ContentRepository::clear_content_genres(&state.db, Some(id), None).await?;
            if !gids.is_empty() {
                ContentRepository::link_movie_genres(&state.db, id, &gids).await?;
            }
        }

        let genres = ContentRepository::get_movie_genres(&state.db, movie.id).await?;
        let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();

        Ok(MovieResponse {
            movie,
            genres: genre_dtos,
        })
    }

    pub async fn delete_movie(state: AppState, id: Uuid) -> Result<()> {
        ContentRepository::delete_movie(&state.db, id).await
    }

    // --- SERIES UPDATES ---

    pub async fn update_series(state: AppState, id: Uuid, req: UpdateSeriesRequest) -> Result<SeriesResponse> {
        let series = ContentRepository::update_series(
            &state.db,
            id,
            req.title,
            req.description,
            req.release_year,
        ).await?;

        if let Some(gids) = req.genre_ids {
            ContentRepository::clear_content_genres(&state.db, None, Some(id)).await?;
             if !gids.is_empty() {
                ContentRepository::link_series_genres(&state.db, id, &gids).await?;
            }
        }

        let genres = ContentRepository::get_series_genres(&state.db, series.id).await?;
        let genre_dtos = genres.into_iter().map(GenreResponse::from).collect();

        Ok(SeriesResponse {
            series,
            genres: genre_dtos,
            seasons: vec![], // TODO: fetch seasons if needed, or keeping lightweight for update
        })
    }

    pub async fn delete_series(state: AppState, id: Uuid) -> Result<()> {
        ContentRepository::delete_series(&state.db, id).await
    }

    // --- SEASON UPDATES ---

    pub async fn update_season(state: AppState, id: Uuid, req: UpdateSeasonRequest) -> Result<SeasonResponse> {
        let season = ContentRepository::update_season(
            &state.db,
            id,
            req.title,
            req.season_number
        ).await?;
        
        // Fetch episodes
        let episodes = ContentRepository::get_season_episodes(&state.db, season.id).await?;

        Ok(SeasonResponse {
            season,
            episodes
        })
    }

    pub async fn delete_season(state: AppState, id: Uuid) -> Result<()> {
        ContentRepository::delete_season(&state.db, id).await
    }
}
