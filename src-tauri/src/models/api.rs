use serde::{Deserialize, Serialize};

use crate::models::app;

#[derive(Deserialize)]
pub enum Events {
    ChatMessageEvent(ChatMessageEvent),
}

#[derive(Deserialize)]
pub struct ChatMessageEvent {
    pub event: String,
    pub data: String,
    pub channel: String,
}

#[derive(Deserialize, Clone)]
pub struct ChatMessageData {
    pub id: String,
    pub chatroom_id: i32,
    pub content: String,
    pub r#type: String,
    pub created_at: String,
    pub sender: Sender,
    pub metadata: Option<MessageRef>,
}

#[derive(PartialEq, Eq, Hash, Deserialize, Clone)]
pub struct Sender {
    pub id: i32,
    pub username: String,
    pub slug: String,
    pub identity: Identity,
}

#[derive(PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct Identity {
    pub color: String,
    pub badges: Vec<Option<Badge>>,
}

#[derive(PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct Badge {
    pub r#type: String,
    pub text: String,
    pub count: Option<i32>,
}

#[derive(Deserialize, Clone)]
pub struct MessageRef {
    pub message_ref: String,
}

pub struct Chatroom {
    pub name: String,
    pub id: i32,
}

#[derive(Serialize)]
pub struct SetSubscription {
    pub event: String,
    pub data: ChannelData,
}
#[derive(Serialize)]
pub struct ChannelData {
    pub channel: String,
}

impl Badge {
    pub fn into_app(self) -> app::Badge {
        app::Badge {
            r#type: self.r#type,
            text: self.text,
            count: self.count,
        }
    }
}
