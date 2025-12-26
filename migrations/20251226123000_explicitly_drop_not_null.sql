-- Explicitly drop the NOT NULL constraint that might persist after dropping the PK
ALTER TABLE content_genres ALTER COLUMN movie_id DROP NOT NULL;
ALTER TABLE content_genres ALTER COLUMN series_id DROP NOT NULL;
