-- Create genres table
CREATE TABLE IF NOT EXISTS genres (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed initial genres
INSERT INTO genres (name, slug) VALUES
    ('Action', 'action'),
    ('Adventure', 'adventure'),
    ('Animation', 'animation'),
    ('Comedy', 'comedy'),
    ('Crime', 'crime'),
    ('Documentary', 'documentary'),
    ('Drama', 'drama'),
    ('Family', 'family'),
    ('Fantasy', 'fantasy'),
    ('History', 'history'),
    ('Horror', 'horror'),
    ('Music', 'music'),
    ('Mystery', 'mystery'),
    ('Romance', 'romance'),
    ('Sci-Fi', 'sci-fi'),
    ('Sport', 'sport'),
    ('Thriller', 'thriller'),
    ('War', 'war'),
    ('Western', 'western')
ON CONFLICT (name) DO NOTHING;
