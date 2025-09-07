-- Add migration script here
CREATE TABLE IF NOT EXISTS badges (
    id INTEGER PRIMARY KEY NOT NULL,
    type TEXT NOT NULL,
    text TEXT NOT NULL,
    UNIQUE (type, text)
);