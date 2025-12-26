-- Drop the primary key which incorrectly forces series_id/movie_id to be NOT NULL
ALTER TABLE content_genres DROP CONSTRAINT content_genres_pkey;

-- Add partial unique constraints instead to ensure uniqueness
CREATE UNIQUE INDEX content_genres_movie_unique_idx ON content_genres (movie_id, genre_id) WHERE movie_id IS NOT NULL;
CREATE UNIQUE INDEX content_genres_series_unique_idx ON content_genres (series_id, genre_id) WHERE series_id IS NOT NULL;
