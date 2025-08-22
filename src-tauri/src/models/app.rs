#![allow(dead_code)] 

use serde::Serialize;
use crate::models::api::{ApiChatMessage, ApiSender, ApiIdentity,ApiChannel, ApiUser, ApiChatroom};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppChatMessage {
    pub id: String,
    pub chatroom_id: u64,
    pub content: String,
    pub message_type: String,
    pub created_at: String,
    pub sender: AppSender,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSender {
    pub id: u64,
    pub username: String,
    pub slug: String,
    pub identity: AppIdentity,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppIdentity {
    pub color: String,
    pub badges: Vec<serde_json::Value>,
}
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppUser {
    pub id: u64,
    pub username: String,
    pub bio: String,
    pub profile_pic: String,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppChatroom {
    pub id: u64,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppChannel {
    pub id: u64,
    pub slug: String,
    pub user: AppUser,
    pub chatroom: AppChatroom,
}

// Conversões
impl From<ApiUser> for AppUser {
    fn from(api_user: ApiUser) -> Self {
        Self {
            id: api_user.id,
            bio: api_user.bio,
            profile_pic: api_user.profile_pic,
            username: api_user.username
        }
    }
}
impl From<ApiChatroom> for AppChatroom {
    fn from(api_chatroom: ApiChatroom) -> Self {
        Self { 
            id: api_chatroom.id,
        }
    }
}
impl From<ApiChannel> for AppChannel {
    fn from(api_channel: ApiChannel) -> Self {
        Self {
            id: api_channel.id,
            slug: api_channel.slug,
            user: api_channel.user.into(),
            chatroom: api_channel.chatroom.into(),
        }
    }
}

impl From<ApiChatMessage> for AppChatMessage {
    fn from(api_message: ApiChatMessage) -> Self {
        AppChatMessage {
            id: api_message.id,
            chatroom_id: api_message.chatroom_id,
            content: api_message.content,
            message_type: api_message.message_type,
            created_at: api_message.created_at,
            sender: api_message.sender.into(), // .into() chama a conversão de ApiSender para AppSender
        }
    }
}

impl From<ApiSender> for AppSender {
    fn from(api_sender: ApiSender) -> Self {
        AppSender {
            id: api_sender.id,
            username: api_sender.username,
            slug: api_sender.slug,
            identity: api_sender.identity.into(), // .into() chama a conversão de ApiIdentity para AppIdentity
        }
    }
}

impl From<ApiIdentity> for AppIdentity {
    fn from(api_identity: ApiIdentity) -> Self {
        AppIdentity {
            color: api_identity.color,
            badges: api_identity.badges,
        }
    }
}