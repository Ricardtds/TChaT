CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    chatroom_id INTEGER NOT NULL,
    content TEXT,
    created_at TEXT,
    sender_id INTEGER,
    FOREIGN KEY(sender_id) REFERENCES senders(id)
);