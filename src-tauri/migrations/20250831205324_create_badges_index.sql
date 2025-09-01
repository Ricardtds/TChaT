-- Add migration script here
CREATE UNIQUE INDEX idx_badges_unique_with_count
ON badges(type, text, count)
WHERE count IS NOT NULL;

CREATE UNIQUE INDEX idx_badges_unique_without_count
ON badges(type, text)
WHERE count IS NULL;