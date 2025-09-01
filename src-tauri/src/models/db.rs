#![allow(dead_code)]
use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct MessageWithSender {
    // Fields from the 'messages' table
    pub id: String,
    pub chatroom_id: i64,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub created_at: String,

    // Fields from the 'senders' table
    pub sender_id: i64,
    pub username: String,
    pub slug: String,
    pub color: String,
}

use serde::Deserialize;

#[derive(Deserialize)]
pub struct SenderData {
    pub username: String,
    pub slug: String,
    pub color: String,
}

#[derive(Deserialize)]
pub struct BadgeData {
    pub r#type: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct NewMessageData {
    pub chatroom_id: i64,
    pub content: String,
    pub r#type: String,
    pub sender: SenderData,
    pub badges: Vec<BadgeData>,
}
