#![allow(dead_code)]
use crate::models::api;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub chatroom_id: i64,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub created_at: String,
    pub sender: Sender,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageWithMetadata {
    pub id: String,
    pub chatroom_id: i64,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub created_at: String,
    pub sender: Sender,
    pub metadata: Metadata,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    message_ref: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Sender {
    pub id: i64,
    pub username: String,
    pub slug: String,
    pub identity: Identity,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Identity {
    pub color: String,
    pub badges: Option<Vec<Badge>>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    #[serde(rename = "type")]
    pub badge_type: String,
    pub text: String,
    pub count: Option<i64>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Chatroom {
    pub id: i32,
}

// Conversões
impl From<api::Chatroom> for Chatroom {
    fn from(api_chatroom: api::Chatroom) -> Self {
        Self {
            id: api_chatroom.id,
        }
    }
}

impl From<api::ChatMessageWithMetadata> for ChatMessageWithMetadata {
    fn from(api_message: api::ChatMessageWithMetadata) -> Self {
        ChatMessageWithMetadata {
            id: api_message.id,
            chatroom_id: api_message.chatroom_id,
            content: api_message.content,
            message_type: api_message.message_type,
            created_at: api_message.created_at,
            sender: api_message.sender.into(),
            metadata: api_message.metadata.into(),
        }
    }
}

impl From<api::ChatMessage> for ChatMessage {
    fn from(api_message: api::ChatMessage) -> Self {
        ChatMessage {
            id: api_message.id,
            chatroom_id: api_message.chatroom_id,
            content: api_message.content,
            message_type: api_message.message_type,
            created_at: api_message.created_at,
            sender: api_message.sender.into(),
        }
    }
}

impl From<api::Metadata> for Metadata {
    fn from(api_metadata: api::Metadata) -> Self {
        Metadata {
            message_ref: api_metadata.message_ref,
        }
    }
}

impl From<api::Sender> for Sender {
    fn from(api_sender: api::Sender) -> Self {
        Sender {
            id: api_sender.id,
            username: api_sender.username,
            slug: api_sender.slug,
            identity: api_sender.identity.into(),
        }
    }
}

impl From<api::Identity> for Identity {
    fn from(value: api::Identity) -> Self {
        Identity {
            color: value.color,
            badges: value
                .badges
                .map(|badges| badges.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<api::Badge> for Badge {
    fn from(value: api::Badge) -> Self {
        Badge {
            badge_type: value.badge_type,
            text: value.text,
            count: value.count,
        }
    }
}
