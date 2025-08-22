#![allow(dead_code)] // Desabilita avisos de código não utilizado neste arquivo

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ApiOuterMessage {
    pub event: String,
    pub data: String,
    pub channel: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ApiChatMessage {
    pub id: String,
    pub chatroom_id: u64,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub created_at: String,
    pub sender: ApiSender,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ApiSender {
    pub id: u64,
    pub username: String,
    pub slug: String,
    pub identity: ApiIdentity,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ApiIdentity {
    pub color: String,
    pub badges: Vec<serde_json::Value>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ApiUser {
    pub id: u64,
    pub username: String,
    pub bio: String,
    pub profile_pic: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ApiChatroom {
    pub id: u64,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ApiChannel {
    pub id: u64,
    pub slug: String,
    pub user: ApiUser,
    pub chatroom: ApiChatroom,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiV2Channel {
    pub id: u64,
    pub slug: String,
    pub chatroom: ApiChatroom,
}