-- Create Movies Table
CREATE TABLE IF NOT EXISTS movies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    video_url TEXT, -- Path to MinIO
    thumbnail_url TEXT, -- Path to MinIO
    release_year INT,
    duration_seconds INT,
    rating FLOAT DEFAULT 0.0,
    views INT DEFAULT 0,
    status VARCHAR(50) DEFAULT 'DRAFT', -- DRAFT, PROCESSING, READY, FAILED
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create Series Table
CREATE TABLE IF NOT EXISTS series (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    thumbnail_url TEXT, -- Path to MinIO
    release_year INT,
    rating FLOAT DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create Seasons Table
CREATE TABLE IF NOT EXISTS seasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    series_id UUID NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    season_number INT NOT NULL,
    title VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(series_id, season_number)
);

-- Create Episodes Table
CREATE TABLE IF NOT EXISTS episodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    season_id UUID NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    episode_number INT NOT NULL,
    title VARCHAR(255),
    description TEXT,
    video_url TEXT, -- Path to MinIO
    thumbnail_url TEXT, -- Path to MinIO
    duration_seconds INT,
    views INT DEFAULT 0,
    status VARCHAR(50) DEFAULT 'DRAFT', -- DRAFT, PROCESSING, READY, FAILED
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(season_id, episode_number)
);

-- Create Content Genres Link Table
CREATE TABLE IF NOT EXISTS content_genres (
    genre_id UUID NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    movie_id UUID REFERENCES movies(id) ON DELETE CASCADE,
    series_id UUID REFERENCES series(id) ON DELETE CASCADE,
    PRIMARY KEY (genre_id, movie_id, series_id),
    CONSTRAINT check_content_type CHECK (
        (movie_id IS NOT NULL AND series_id IS NULL) OR
        (movie_id IS NULL AND series_id IS NOT NULL)
    )
);

-- Indexes for performance
CREATE INDEX idx_movies_slug ON movies(slug);
CREATE INDEX idx_series_slug ON series(slug);
CREATE INDEX idx_episodes_season_id ON episodes(season_id);
CREATE INDEX idx_seasons_series_id ON seasons(series_id);
CREATE INDEX idx_content_genres_genre_id ON content_genres(genre_id);
