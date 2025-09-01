-- Add migration script here
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    chatroom_id INTEGER NOT NULL,
    sender_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    metadata TEXT,
    FOREIGN KEY (chatroom_id) REFERENCES chatrooms (id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES senders (id)
);