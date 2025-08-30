use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]

pub struct MessageWithSender {
    // Fields from the 'messages' table
    pub message_id: String,
    pub chatroom_id: i64,
    pub content: Option<String>,
    pub created_at: String,

    // Fields from the 'senders' table
    pub sender_id: i64,
    pub username: Option<String>,
    pub slug: Option<String>,
    pub color: Option<String>,
}