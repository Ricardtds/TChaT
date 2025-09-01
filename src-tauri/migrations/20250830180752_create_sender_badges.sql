-- Add migration script here
CREATE TABLE IF NOT EXISTS sender_badges (
    sender_id INTEGER NOT NULL,
    badge_id INTEGER NOT NULL,
    chatroom_id INTEGER NOT NULL,
    PRIMARY KEY (sender_id, badge_id, chatroom_id),
    FOREIGN KEY (chatroom_id) REFERENCES chatrooms (id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES senders (id),
    FOREIGN KEY (badge_id) REFERENCES badges (id)
);