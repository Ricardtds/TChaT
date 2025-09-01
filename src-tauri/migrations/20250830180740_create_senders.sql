-- Add migration script here
CREATE TABLE IF NOT EXISTS senders (
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL,
    color TEXT NOT NULL
);