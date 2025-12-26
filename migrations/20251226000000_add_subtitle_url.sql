-- Add subtitle_url to movies table
ALTER TABLE movies ADD COLUMN subtitle_url VARCHAR(255);

-- Add subtitle_url to episodes table
ALTER TABLE episodes ADD COLUMN subtitle_url VARCHAR(255);
