use serde::Deserialize;

pub struct Message {
    pub id: String,
    pub chatroom_id: i32,
    pub sender_id: i32,
    pub content: String,
    pub r#type: String,
    pub created_at: String,
    pub metadata: Option<String>
}

#[derive(Deserialize)]pub struct SenderBadge {
    pub chatroom_id: i32,
    pub sender_id: i32,
    pub badge_id: i32,
    pub count: Option<i32>
}