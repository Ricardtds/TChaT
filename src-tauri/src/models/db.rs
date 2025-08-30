#[derive(Debug)]
pub struct Sender {
    pub id: u64,
    pub username: String,
    pub slug: String,
    pub color: String,
}

#[derive(Debug)]
pub struct ChatMessage {
    pub id: String,
    pub chatroom_id: u64,
    pub content: Option<String>,
    pub created_at: String,
    pub sender_id: u64
}